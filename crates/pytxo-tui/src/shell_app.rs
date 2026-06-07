use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use pytxo_core::PermissionProfile;
use pytxo_core::RunId;
use pytxo_orchestrate::{dashboard_snapshot, hitl_respond, DashboardSnapshot};
use pytxo_shell::{complete_line, parse_line, ShellEvent, ShellInput, ShellSession};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::DefaultTerminal;

use crate::panels::board::{self, BoardView};
use crate::panels::prompt::Prompt;
use crate::panels::scrollback::Scrollback;
use crate::panels::trust::TrustModal;

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

pub fn run() -> anyhow::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = ratatui::init();
    let rt = tokio::runtime::Runtime::new()?;
    let result = run_loop(&mut terminal, &rt);
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
    session: ShellSession,
    scrollback: Scrollback,
    prompt: Prompt,
    board: BoardView,
    scroll_offset: usize,
    status_message: String,
    last_refresh: Instant,
    active_run_label: Option<String>,
    trust_tier: Option<PermissionProfile>,
    phase: AppPhase,
}

impl ShellApp {
    fn new(_rt: &tokio::runtime::Runtime) -> anyhow::Result<Self> {
        let snapshot = dashboard_snapshot(None, 12)?;
        let session = ShellSession::new(None, None)?;
        let trusted = session.is_trusted();
        let repo_path = session.repo.display().to_string();
        let phase = if trusted {
            AppPhase::Shell
        } else {
            AppPhase::Trust(TrustModal::new(repo_path))
        };
        let mut app = Self {
            snapshot,
            session,
            scrollback: Scrollback::new(500),
            prompt: Prompt::new(),
            board: BoardView {
                run_selected: 0,
                hitl_selected: 0,
            },
            scroll_offset: 0,
            status_message: String::new(),
            last_refresh: Instant::now(),
            active_run_label: None,
            trust_tier: None,
            phase,
        };
        app.scrollback
            .push("Hypervisor Shell — /help for commands · /models search …");
        Ok(app)
    }

    fn refresh_board(&mut self) {
        match dashboard_snapshot(None, 12) {
            Ok(s) => {
                self.snapshot = s;
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
            }
            Err(e) => self.status_message = format!("Refresh failed: {e}"),
        }
        self.last_refresh = Instant::now();
    }

    fn apply_events(&mut self, events: Vec<ShellEvent>) {
        for ev in events {
            match ev {
                ShellEvent::Output(s) => self.scrollback.push(&s),
                ShellEvent::PlanPreview(json) => self.scrollback.push(&json),
                ShellEvent::RunStarted(id) => {
                    self.active_run_label = Some(id.0.clone());
                    self.scrollback.push(&format!("Run started: {}", id.0));
                }
                ShellEvent::RunFinished(id) => {
                    self.scrollback.push(&format!("Run finished: {}", id.0));
                    self.active_run_label = None;
                }
                ShellEvent::Error(e) => {
                    self.scrollback.push(&format!("error: {e}"));
                    self.status_message = e;
                }
            }
        }
    }

    async fn submit_line(&mut self, line: String) {
        self.prompt.push_history(line.clone());
        self.scrollback.push(&format!("> {line}"));
        let input = parse_line(&line);
        if matches!(input, ShellInput::ReplExit) {
            self.status_message = "exit".into();
            return;
        }
        let events = self.session.handle(input).await;
        self.apply_events(events);
        self.refresh_board();
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
}

fn run_loop(terminal: &mut DefaultTerminal, rt: &tokio::runtime::Runtime) -> anyhow::Result<()> {
    let mut app = ShellApp::new(rt)?;
    let mut should_exit = false;

    loop {
        if matches!(app.phase, AppPhase::Shell) && app.last_refresh.elapsed() >= REFRESH_INTERVAL {
            app.refresh_board();
        }

        terminal.draw(|frame| {
            let area = frame.area();
            if let AppPhase::Trust(ref modal) = app.phase {
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
                &app.snapshot,
                app.active_run_label.as_deref(),
                app.tier_label(),
                &app.session.config.agent,
                &app.board,
            );
            app.scrollback.draw(frame, chunks[1], app.scroll_offset);
            app.prompt.draw(frame, chunks[2], &app.status_message);
        })?;

        if should_exit || app.status_message == "exit" {
            break;
        }

        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if let AppPhase::Trust(ref mut modal) = app.phase {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => should_exit = true,
                        KeyCode::Up => modal.move_up(),
                        KeyCode::Down => modal.move_down(),
                        KeyCode::Enter => match modal.accept(&app.session.repo) {
                            Ok(tier) => {
                                app.trust_tier = Some(tier);
                                app.phase = AppPhase::Shell;
                                app.session =
                                    ShellSession::new(None, Some(app.session.repo.clone()))?;
                                app.scrollback.push("Folder trusted — /dry-run then /run");
                            }
                            Err(e) => app.status_message = e.to_string(),
                        },
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') if app.prompt.buffer.is_empty() => should_exit = true,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let events = rt.block_on(app.session.handle(ShellInput::Slash(
                            pytxo_shell::SlashCommand::Stop { all: false },
                        )));
                        app.apply_events(events);
                        should_exit = true;
                    }
                    KeyCode::Enter => {
                        let line = std::mem::take(&mut app.prompt.buffer);
                        if line.trim().eq_ignore_ascii_case("exit")
                            || line.trim().eq_ignore_ascii_case("quit")
                        {
                            should_exit = true;
                        } else {
                            rt.block_on(app.submit_line(line));
                        }
                    }
                    KeyCode::Backspace => {
                        app.prompt.buffer.pop();
                    }
                    KeyCode::Up if app.prompt.buffer.is_empty() => app.prompt.history_up(),
                    KeyCode::Down if app.prompt.buffer.is_empty() => app.prompt.history_down(),
                    KeyCode::Up if !app.prompt.buffer.is_empty() => {
                        app.board.run_selected = app.board.run_selected.saturating_sub(1);
                    }
                    KeyCode::Down
                        if !app.prompt.buffer.is_empty()
                            && app.board.run_selected + 1 < app.snapshot.runs.len() =>
                    {
                        app.board.run_selected += 1;
                    }
                    KeyCode::Up => app.scroll_offset = app.scroll_offset.saturating_add(1),
                    KeyCode::Down => app.scroll_offset = app.scroll_offset.saturating_sub(1),
                    KeyCode::Tab if !app.prompt.buffer.is_empty() => {
                        if let Some(completed) = complete_line(&app.prompt.buffer) {
                            app.prompt.buffer = completed;
                        }
                    }
                    KeyCode::Tab if !app.snapshot.hitl_pending.is_empty() => {
                        app.board.hitl_selected =
                            (app.board.hitl_selected + 1) % app.snapshot.hitl_pending.len();
                    }
                    KeyCode::Char('a') if app.prompt.buffer.is_empty() => {
                        if let Some(req) = app.snapshot.hitl_pending.get(app.board.hitl_selected) {
                            match hitl_respond(None, &req.id, true) {
                                Ok(true) => app.status_message = format!("Approved {}", req.id),
                                Ok(false) => app.status_message = "Request not pending".into(),
                                Err(e) => app.status_message = format!("Approve failed: {e}"),
                            }
                            app.refresh_board();
                        }
                    }
                    KeyCode::Char('x') if app.prompt.buffer.is_empty() => {
                        if let Some(req) = app.snapshot.hitl_pending.get(app.board.hitl_selected) {
                            match hitl_respond(None, &req.id, false) {
                                Ok(true) => app.status_message = format!("Denied {}", req.id),
                                Ok(false) => app.status_message = "Request not pending".into(),
                                Err(e) => app.status_message = format!("Deny failed: {e}"),
                            }
                            app.refresh_board();
                        }
                    }
                    KeyCode::Char(c) => app.prompt.buffer.push(c),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
