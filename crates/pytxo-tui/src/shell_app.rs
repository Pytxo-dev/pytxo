use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use pytxo_core::PermissionProfile;
use pytxo_core::RunId;
use pytxo_orchestrate::{
    dashboard_snapshot_light, effective_entitlements, hitl_respond, project_roots, run_doctor,
    DashboardSnapshot, DoctorReport, EntitlementStatus,
};
use pytxo_shell::{complete_line, parse_line, ShellEvent, ShellInput, ShellSession};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::DefaultTerminal;

use crate::input::{accepts_key_event, is_cancel_key, is_submit_key};
use crate::panels::board::{self, BoardView};
use crate::panels::prompt::Prompt;
use crate::panels::scrollback::Scrollback;
use crate::panels::splash;
use crate::panels::trust::TrustModal;

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);
const DOCTOR_INTERVAL: Duration = Duration::from_secs(60);
const POLL_ACTIVE: Duration = Duration::from_millis(120);
const POLL_IDLE: Duration = Duration::from_millis(250);

/// Run the TUI. When called from `#[tokio::main]`, runs on a dedicated thread so
/// the CLI runtime never nests with the TUI's own `current_thread` runtime.
pub fn run() -> anyhow::Result<()> {
    if tokio::runtime::Handle::try_current().is_ok() {
        std::thread::Builder::new()
            .name("pytxo-tui".into())
            .spawn(run_on_thread)
            .map_err(|e| anyhow::anyhow!("failed to spawn tui thread: {e}"))?
            .join()
            .map_err(|_| anyhow::anyhow!("tui thread panicked"))?
    } else {
        run_on_thread()
    }
}

fn run_on_thread() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run_async())
}

async fn run_async() -> anyhow::Result<()> {
    if std::env::var("PYTXO_TUI_INSTANT_EXIT").ok().as_deref() == Some("1") {
        return Ok(());
    }
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal).await;
    ratatui::restore();
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    result
}

enum AppPhase {
    Trust(TrustModal),
    Shell,
}

struct ShellApp {
    snapshot: DashboardSnapshot,
    doctor_cache: DoctorReport,
    session: ShellSession,
    scrollback: Scrollback,
    prompt: Prompt,
    board: BoardView,
    scroll_offset: usize,
    status_message: String,
    last_refresh: Instant,
    last_doctor_refresh: Instant,
    active_run_label: Option<String>,
    trust_tier: Option<PermissionProfile>,
    org_policy_label: Option<String>,
    phase: AppPhase,
    needs_redraw: bool,
    show_splash: bool,
    /// First paint uses light snapshot only; full doctor runs on the next refresh cycle.
    doctor_boot_deferred: bool,
}

fn trust_folders(repo: &Path) -> Vec<String> {
    let repo_str = repo.display().to_string();
    match project_roots(None, None) {
        Ok(roots) if roots.len() > 1 => roots.into_iter().map(|(_, path, _, _, _)| path).collect(),
        _ => vec![repo_str],
    }
}

impl ShellApp {
    fn new() -> anyhow::Result<Self> {
        let snapshot = dashboard_snapshot_light(None, 12)?;
        let session = ShellSession::new(None, None)?;
        let trusted = session.is_trusted();
        let folders = trust_folders(&session.repo);
        let phase = if trusted {
            AppPhase::Shell
        } else {
            AppPhase::Trust(TrustModal::new(folders))
        };
        let now = Instant::now();
        let mut app = Self {
            snapshot,
            doctor_cache: DoctorReport { checks: vec![] },
            session,
            scrollback: Scrollback::new(500),
            prompt: Prompt::new(),
            board: BoardView {
                run_selected: 0,
                hitl_selected: 0,
            },
            scroll_offset: 0,
            status_message: String::new(),
            last_refresh: now,
            last_doctor_refresh: now,
            active_run_label: None,
            trust_tier: None,
            org_policy_label: None,
            phase,
            needs_redraw: true,
            show_splash: true,
            doctor_boot_deferred: false,
        };
        app.scrollback
            .push("Hypervisor Shell — /help · Discord: https://discord.gg/AUFRPFjSYv");
        Ok(app)
    }

    fn dismiss_splash(&mut self) {
        if self.show_splash {
            self.show_splash = false;
            self.needs_redraw = true;
        }
    }

    fn splash_visible(&self) -> bool {
        self.show_splash && self.scrollback.len() <= 1
    }

    fn refresh_board(&mut self, include_doctor: bool) {
        if include_doctor {
            match run_doctor(None) {
                Ok(d) => {
                    self.doctor_cache = d;
                    self.last_doctor_refresh = Instant::now();
                }
                Err(e) => self.status_message = format!("Doctor failed: {e}"),
            }
        }
        match dashboard_snapshot_light(None, 12) {
            Ok(s) => {
                self.snapshot = s;
                if let Ok(ent) = effective_entitlements(&self.session.config) {
                    self.org_policy_label = org_policy_hint(&ent);
                }
                if self.board.run_selected >= self.snapshot.runs.len() {
                    self.board.run_selected = self.snapshot.runs.len().saturating_sub(1);
                }
                if self.board.hitl_selected >= self.snapshot.hitl_pending.len() {
                    self.board.hitl_selected = self.snapshot.hitl_pending.len().saturating_sub(1);
                }
                if let Some(ref id) = self.active_run_label {
                    if let Some(run) = self.snapshot.runs.iter().find(|r| r.id == *id) {
                        if run.status == "completed" || run.status == "failed" {
                            let finished = RunId(id.clone());
                            self.active_run_label = None;
                            self.session.active_run = None;
                            self.apply_events(vec![ShellEvent::RunFinished(finished)]);
                        }
                    }
                }
                self.needs_redraw = true;
            }
            Err(e) => self.status_message = format!("Refresh failed: {e}"),
        }
        self.last_refresh = Instant::now();
    }

    fn maybe_refresh(&mut self) {
        // Cold path: never block first paint on full doctor (PTY smoke + git + HTTP).
        if !self.doctor_boot_deferred {
            self.refresh_board(false);
            self.doctor_boot_deferred = true;
            return;
        }
        if self.doctor_cache.checks.is_empty()
            || self.last_doctor_refresh.elapsed() >= DOCTOR_INTERVAL
        {
            self.refresh_board(true);
        } else if self.last_refresh.elapsed() >= REFRESH_INTERVAL {
            self.refresh_board(false);
        }
    }

    fn apply_events(&mut self, events: Vec<ShellEvent>) {
        for ev in events {
            match ev {
                ShellEvent::Output(s) => {
                    self.push_scrollback(&s);
                    self.needs_redraw = true;
                }
                ShellEvent::PlanPreview(json) => {
                    self.push_scrollback(&json);
                    self.needs_redraw = true;
                }
                ShellEvent::RunStarted(id) => {
                    self.active_run_label = Some(id.0.clone());
                    self.push_scrollback(&format!("Run started: {}", id.0));
                    self.needs_redraw = true;
                }
                ShellEvent::RunFinished(id) => {
                    self.push_scrollback(&format!("Run finished: {}", id.0));
                    self.active_run_label = None;
                    self.needs_redraw = true;
                }
                ShellEvent::Error(e) => {
                    self.push_scrollback(&format!("error: {e}"));
                    self.status_message = e;
                    self.needs_redraw = true;
                }
            }
        }
    }

    /// Push scrollback; auto-follow bottom when not scrolled up.
    fn push_scrollback(&mut self, text: &str) {
        let following = self.scroll_offset == 0;
        self.scrollback.push(text);
        if following {
            self.scroll_offset = 0;
        }
    }

    async fn submit_line(&mut self, line: String) {
        if line.trim_start().starts_with('/') {
            self.dismiss_splash();
        }
        self.prompt.push_history(line.clone());
        self.scroll_offset = 0;
        self.push_scrollback(&format!("> {line}"));
        let input = parse_line(&line);
        if matches!(input, ShellInput::ReplExit) {
            self.status_message = "exit".into();
            return;
        }
        let events = self.session.handle(input).await;
        self.apply_events(events);
        self.refresh_board(false);
    }

    fn tier_label(&self) -> &'static str {
        if let Some(t) = self.trust_tier {
            return match t {
                PermissionProfile::DeepSpace => "deep_space",
                PermissionProfile::Orbit => "orbit",
                PermissionProfile::Galaxy => "galaxy",
                PermissionProfile::Supernova => "supernova",
            };
        }
        if self.session.is_trusted() {
            "trusted"
        } else {
            "untrusted"
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        if let AppPhase::Trust(ref modal) = self.phase {
            modal.draw(frame, area);
            return;
        }
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(12),
                Constraint::Min(6),
                Constraint::Length(4),
            ])
            .split(area);
        board::draw(
            frame,
            chunks[0],
            &self.snapshot,
            &self.doctor_cache,
            self.active_run_label.as_deref(),
            self.tier_label(),
            self.org_policy_label.as_deref(),
            &self.session.config.agent,
            &self.board,
        );
        if self.splash_visible() {
            splash::draw(frame, chunks[1]);
        } else {
            self.scrollback.draw(frame, chunks[1], self.scroll_offset);
        }
        self.prompt.draw(
            frame,
            chunks[2],
            &self.status_message,
            !self.snapshot.hitl_pending.is_empty(),
            false,
        );
    }
}

fn org_policy_hint(ent: &EntitlementStatus) -> Option<String> {
    let ceiling = ent.permission_ceiling?;
    let org = ent.org_id.as_deref().unwrap_or("org");
    let profile = match ceiling {
        PermissionProfile::DeepSpace => "deepspace",
        PermissionProfile::Orbit => "orbit",
        PermissionProfile::Galaxy => "galaxy",
        PermissionProfile::Supernova => "supernova",
    };
    Some(format!("org:{org} cap:{profile}"))
}

async fn run_loop(terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    let mut app = ShellApp::new()?;
    let mut should_exit = false;

    loop {
        app.maybe_refresh();

        if app.needs_redraw {
            terminal.draw(|frame| app.draw(frame))?;
            app.needs_redraw = false;
        }

        if should_exit || app.status_message == "exit" {
            break;
        }

        let poll = if app.active_run_label.is_some() {
            POLL_ACTIVE
        } else {
            POLL_IDLE
        };

        if event::poll(poll)? {
            if let Event::Key(key) = event::read()? {
                if !accepts_key_event(&key) {
                    continue;
                }

                if let AppPhase::Trust(ref mut modal) = app.phase {
                    app.needs_redraw = true;
                    if is_cancel_key(key.code) {
                        should_exit = true;
                    } else if matches!(key.code, KeyCode::Up) {
                        modal.move_up();
                    } else if matches!(key.code, KeyCode::Down) {
                        modal.move_down();
                    } else if is_submit_key(key.code) {
                        modal.begin_accept();
                        match modal.accept(&app.session.repo) {
                            Ok(tier) => {
                                app.trust_tier = Some(tier);
                                match ShellSession::new(None, Some(app.session.repo.clone())) {
                                    Ok(session) => {
                                        app.session = session;
                                        app.phase = AppPhase::Shell;
                                        let tier_name = app.tier_label();
                                        app.scroll_offset = 0;
                                        app.push_scrollback(&format!(
                                            "✓ Folder trusted ({tier_name}) — /dry-run then /run"
                                        ));
                                    }
                                    Err(e) => modal.set_error(e.to_string()),
                                }
                            }
                            Err(e) => modal.set_error(e.to_string()),
                        }
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') if app.prompt.buffer.is_empty() => should_exit = true,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.dismiss_splash();
                        let events = app
                            .session
                            .handle(ShellInput::Slash(pytxo_shell::SlashCommand::Stop {
                                all: false,
                            }))
                            .await;
                        app.apply_events(events);
                        should_exit = true;
                    }
                    code if is_submit_key(code) => {
                        app.dismiss_splash();
                        let line = std::mem::take(&mut app.prompt.buffer);
                        if line.trim().eq_ignore_ascii_case("exit")
                            || line.trim().eq_ignore_ascii_case("quit")
                        {
                            should_exit = true;
                        } else {
                            app.submit_line(line).await;
                        }
                    }
                    KeyCode::Backspace => {
                        app.dismiss_splash();
                        app.prompt.buffer.pop();
                        app.needs_redraw = true;
                    }
                    KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.dismiss_splash();
                        app.scroll_offset = app.scroll_offset.saturating_add(1);
                        app.needs_redraw = true;
                    }
                    KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.dismiss_splash();
                        app.scroll_offset = app.scroll_offset.saturating_sub(1);
                        app.needs_redraw = true;
                    }
                    KeyCode::Up if app.prompt.buffer.is_empty() => {
                        app.prompt.history_up();
                        app.needs_redraw = true;
                    }
                    KeyCode::Down if app.prompt.buffer.is_empty() => {
                        app.prompt.history_down();
                        app.needs_redraw = true;
                    }
                    KeyCode::Up if !app.prompt.buffer.is_empty() => {
                        app.board.run_selected = app.board.run_selected.saturating_sub(1);
                        app.needs_redraw = true;
                    }
                    KeyCode::Down
                        if !app.prompt.buffer.is_empty()
                            && app.board.run_selected + 1 < app.snapshot.runs.len() =>
                    {
                        app.board.run_selected += 1;
                        app.needs_redraw = true;
                    }
                    KeyCode::Tab if !app.prompt.buffer.is_empty() => {
                        if let Some(completed) = complete_line(&app.prompt.buffer) {
                            app.prompt.buffer = completed;
                            app.needs_redraw = true;
                        }
                    }
                    KeyCode::Tab if !app.snapshot.hitl_pending.is_empty() => {
                        app.board.hitl_selected =
                            (app.board.hitl_selected + 1) % app.snapshot.hitl_pending.len();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('a') if app.prompt.buffer.is_empty() => {
                        if let Some(req) = app.snapshot.hitl_pending.get(app.board.hitl_selected) {
                            match hitl_respond(None, &req.id, true) {
                                Ok(true) => app.status_message = format!("Approved {}", req.id),
                                Ok(false) => app.status_message = "Request not pending".into(),
                                Err(e) => app.status_message = format!("Approve failed: {e}"),
                            }
                            app.refresh_board(false);
                        }
                    }
                    KeyCode::Char('x') if app.prompt.buffer.is_empty() => {
                        if let Some(req) = app.snapshot.hitl_pending.get(app.board.hitl_selected) {
                            match hitl_respond(None, &req.id, false) {
                                Ok(true) => app.status_message = format!("Denied {}", req.id),
                                Ok(false) => app.status_message = "Request not pending".into(),
                                Err(e) => app.status_message = format!("Deny failed: {e}"),
                            }
                            app.refresh_board(false);
                        }
                    }
                    KeyCode::Char(c) => {
                        app.dismiss_splash();
                        app.prompt.buffer.push(c);
                        app.needs_redraw = true;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run;
    use pytxo_shell::{parse_line, ShellEvent, ShellSession};

    #[tokio::test]
    async fn submit_help_does_not_nested_block_on() {
        let mut session = ShellSession::new(None, Some(std::env::current_dir().unwrap())).unwrap();
        let _ = pytxo_orchestrate::trust_repo(&session.repo, pytxo_core::PermissionProfile::Orbit);
        let events = session.handle(parse_line("/help")).await;
        assert!(events
            .iter()
            .any(|e| { matches!(e, ShellEvent::Output(s) if s.contains("/dry-run")) }));
    }

    #[tokio::test]
    async fn run_from_tokio_context_does_not_panic() {
        std::env::set_var("PYTXO_TUI_INSTANT_EXIT", "1");
        let result = std::panic::catch_unwind(run);
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    #[tokio::test]
    async fn run_loop_submit_help_via_session() {
        let mut session = ShellSession::new(None, Some(std::env::current_dir().unwrap())).unwrap();
        let _ = pytxo_orchestrate::trust_repo(&session.repo, pytxo_core::PermissionProfile::Orbit);
        let events = session.handle(parse_line("/help")).await;
        assert!(events
            .iter()
            .any(|e| matches!(e, ShellEvent::Output(s) if s.contains("/agents"))));
    }
}
