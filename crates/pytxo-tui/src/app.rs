use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use pytxo_orchestrate::{dashboard_snapshot, hitl_respond, DashboardSnapshot, DoctorReport};
use pytxo_runner::HitlRequest;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap};
use ratatui::DefaultTerminal;

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

pub fn run() -> anyhow::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = ratatui::init();
    let result = run_loop(&mut terminal);
    ratatui::restore();
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    result
}

struct AppState {
    snapshot: DashboardSnapshot,
    run_selected: usize,
    hitl_selected: usize,
    show_doctor: bool,
    show_help: bool,
    status_message: String,
    last_refresh: Instant,
}

impl AppState {
    fn refresh(&mut self) {
        match dashboard_snapshot(None, 12) {
            Ok(s) => {
                self.snapshot = s;
                if self.run_selected >= self.snapshot.runs.len() {
                    self.run_selected = self.snapshot.runs.len().saturating_sub(1);
                }
                if self.hitl_selected >= self.snapshot.hitl_pending.len() {
                    self.hitl_selected = self.snapshot.hitl_pending.len().saturating_sub(1);
                }
                self.status_message = "Refreshed".into();
            }
            Err(e) => self.status_message = format!("Refresh failed: {e}"),
        }
        self.last_refresh = Instant::now();
    }
}

fn run_loop(terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    let mut state = AppState {
        snapshot: dashboard_snapshot(None, 12)?,
        run_selected: 0,
        hitl_selected: 0,
        show_doctor: false,
        show_help: false,
        status_message: String::new(),
        last_refresh: Instant::now(),
    };

    loop {
        if state.last_refresh.elapsed() >= REFRESH_INTERVAL {
            state.refresh();
        }

        terminal.draw(|frame| draw(frame, &state))?;

        if event::poll(Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if state.show_doctor || state.show_help {
                    if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('d') | KeyCode::Char('?')) {
                        state.show_doctor = false;
                        state.show_help = false;
                    }
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('r') => state.refresh(),
                    KeyCode::Char('d') => state.show_doctor = true,
                    KeyCode::Char('?') => state.show_help = true,
                    KeyCode::Up => {
                        state.run_selected = state.run_selected.saturating_sub(1);
                    }
                    KeyCode::Down if state.run_selected + 1 < state.snapshot.runs.len() => {
                        state.run_selected += 1;
                    }
                    KeyCode::Char('a') => {
                        if let Some(req) = state.snapshot.hitl_pending.get(state.hitl_selected) {
                            match hitl_respond(None, &req.id, true) {
                                Ok(true) => state.status_message = format!("Approved {}", req.id),
                                Ok(false) => state.status_message = "Request not pending".into(),
                                Err(e) => state.status_message = format!("Approve failed: {e}"),
                            }
                            state.refresh();
                        }
                    }
                    KeyCode::Char('x') => {
                        if let Some(req) = state.snapshot.hitl_pending.get(state.hitl_selected) {
                            match hitl_respond(None, &req.id, false) {
                                Ok(true) => state.status_message = format!("Denied {}", req.id),
                                Ok(false) => state.status_message = "Request not pending".into(),
                                Err(e) => state.status_message = format!("Deny failed: {e}"),
                            }
                            state.refresh();
                        }
                    }
                    KeyCode::Tab if !state.snapshot.hitl_pending.is_empty() => {
                        state.hitl_selected =
                            (state.hitl_selected + 1) % state.snapshot.hitl_pending.len();
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn draw(frame: &mut ratatui::Frame, state: &AppState) {
    let area = frame.area();
    if state.show_help {
        draw_modal(frame, area, "Help", help_lines());
        return;
    }
    if state.show_doctor {
        draw_doctor_modal(frame, area, &state.snapshot.doctor);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(6),
            Constraint::Length(2),
        ])
        .split(area);

    draw_header(frame, chunks[0], state);
    draw_doctor_strip(frame, chunks[1], &state.snapshot.doctor);
    draw_runs(frame, chunks[2], state);
    draw_hitl(frame, chunks[3], state);
    draw_footer(frame, chunks[4], state);
}

fn theme_border() -> Style {
    Style::default().fg(Color::Rgb(39, 39, 42))
}

fn theme_accent() -> Style {
    Style::default().fg(Color::Rgb(45, 212, 191))
}

fn theme_muted() -> Style {
    Style::default().fg(Color::Rgb(148, 148, 168))
}

fn theme_title() -> Style {
    Style::default()
        .fg(Color::Rgb(167, 139, 250))
        .add_modifier(Modifier::BOLD)
}

fn draw_header(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let doctor_ok = state.snapshot.doctor.all_ok();
    let doctor_style = if doctor_ok {
        Style::default().fg(Color::Rgb(74, 222, 128))
    } else {
        Style::default().fg(Color::Rgb(248, 113, 113))
    };
    let domain_count = state.snapshot.domains.len();
    let text = Line::from(vec![
        Span::styled(" Pytxo ", theme_title()),
        Span::styled(format!("v{} ", state.snapshot.version), theme_accent()),
        Span::styled("│ ", theme_muted()),
        Span::styled(
            truncate(&state.snapshot.repo_root, 48),
            Style::default().fg(Color::White),
        ),
        Span::styled(" │ ", theme_muted()),
        Span::styled(
            if doctor_ok { "doctor ok" } else { "doctor fail" },
            doctor_style,
        ),
        Span::styled(format!(" │ {domain_count} domains"), theme_muted()),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme_border())
        .style(Style::default().bg(Color::Rgb(2, 2, 5)));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_doctor_strip(frame: &mut ratatui::Frame, area: Rect, doctor: &DoctorReport) {
    let failed = doctor.checks.iter().filter(|c| !c.ok).count();
    let summary = if doctor.all_ok() {
        "All doctor checks passed".to_string()
    } else {
        format!("{failed} check(s) failed — press d for details")
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme_border())
        .title(Span::styled(" Doctor ", theme_accent()))
        .style(Style::default().bg(Color::Rgb(10, 10, 15)));
    frame.render_widget(
        Paragraph::new(summary).style(theme_muted()).block(block),
        area,
    );
}

fn draw_runs(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let header = Row::new(vec!["Run ID", "Status", "Agents", "Cost USD"])
        .style(theme_accent())
        .bottom_margin(1);
    let rows: Vec<Row> = state
        .snapshot
        .runs
        .iter()
        .map(|r| {
            Row::new(vec![
                Cell::from(truncate(&r.id, 36)),
                Cell::from(r.status.clone()),
                Cell::from(r.agents.len().to_string()),
                Cell::from(
                    r.estimated_cost_usd
                        .map(|c| format!("{c:.4}"))
                        .unwrap_or_else(|| "—".into()),
                ),
            ])
        })
        .collect();

    let table = Table::new(rows, [
        Constraint::Percentage(45),
        Constraint::Percentage(20),
        Constraint::Percentage(15),
        Constraint::Percentage(20),
    ])
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme_border())
            .title(Span::styled(" Recent runs ", theme_accent()))
            .style(Style::default().bg(Color::Rgb(10, 10, 15))),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(30, 27, 46))
            .add_modifier(Modifier::BOLD),
    );

    let mut table_state = TableState::default().with_selected(Some(state.run_selected));
    frame.render_stateful_widget(table, area, &mut table_state);

    if let Some(run) = state.snapshot.runs.get(state.run_selected) {
        let agent_lines: Vec<Line> = run
            .agents
            .iter()
            .map(|a| {
                Line::from(format!(
                    "  {} wave={} {} exit={:?}",
                    a.id, a.wave, a.status, a.exit_code
                ))
            })
            .collect();
        if !agent_lines.is_empty() && area.height > 6 {
            let detail_area = Rect {
                y: area.y + area.height.saturating_sub(4),
                height: 3,
                ..area
            };
            frame.render_widget(
                Paragraph::new(agent_lines).style(theme_muted()),
                detail_area,
            );
        }
    }
}

fn draw_hitl(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let pending = &state.snapshot.hitl_pending;
    let body = if pending.is_empty() {
        vec![Line::from("No pending approvals")]
    } else {
        pending
            .iter()
            .enumerate()
            .map(|(i, r)| hitl_line(r, i == state.hitl_selected))
            .collect()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme_border())
        .title(Span::styled(" HITL queue ", theme_accent()))
        .style(Style::default().bg(Color::Rgb(10, 10, 15)));
    frame.render_widget(Paragraph::new(body).block(block), area);
}

fn hitl_line(req: &HitlRequest, selected: bool) -> Line<'static> {
    let style = if selected {
        Style::default()
            .fg(Color::Rgb(232, 121, 249))
            .add_modifier(Modifier::BOLD)
    } else {
        theme_muted()
    };
    Line::from(Span::styled(
        format!(" {} [{}] {} — {}", req.id, req.agent_key, req.action, req.reason),
        style,
    ))
}

fn draw_footer(frame: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let msg = if state.status_message.is_empty() {
        "q quit · r refresh · d doctor · Tab HITL · a approve · x deny · ? help".to_string()
    } else {
        state.status_message.clone()
    };
    frame.render_widget(
        Paragraph::new(msg).style(theme_muted()),
        area,
    );
}

fn draw_doctor_modal(frame: &mut ratatui::Frame, area: Rect, doctor: &DoctorReport) {
    let lines: Vec<Line> = doctor
        .checks
        .iter()
        .map(|c| {
            let mark = if c.ok { "✓" } else { "✗" };
            let style = if c.ok {
                Style::default().fg(Color::Rgb(74, 222, 128))
            } else {
                Style::default().fg(Color::Rgb(248, 113, 113))
            };
            Line::from(vec![
                Span::styled(format!("{mark} "), style),
                Span::styled(format!("{} — ", c.name), Style::default().fg(Color::White)),
                Span::raw(c.detail.clone()),
            ])
        })
        .collect();
    draw_modal(frame, area, "Doctor", lines);
}

fn draw_modal(frame: &mut ratatui::Frame, area: Rect, title: &str, lines: Vec<Line<'_>>) {
    let popup = centered_rect(80, 70, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        area,
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme_accent())
        .title(Span::styled(format!(" {title} "), theme_title()))
        .style(Style::default().bg(Color::Rgb(10, 10, 15)));
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true })
            .style(theme_muted()),
        popup,
    );
    let hint = Paragraph::new("Esc or q to close").style(theme_muted());
    frame.render_widget(hint, popup.inner(Margin::new(2, 2)));
}

fn help_lines() -> Vec<Line<'static>> {
    vec![
        Line::from("Pytxo dashboard — local agent hypervisor"),
        Line::from(""),
        Line::from("↑/↓  select run"),
        Line::from("r    refresh snapshot"),
        Line::from("d    doctor details"),
        Line::from("Tab  cycle HITL selection"),
        Line::from("a/x  approve / deny HITL request"),
        Line::from("q    quit"),
        Line::from(""),
        Line::from("Subcommands: pytxo run, doctor, status, logs, …"),
    ]
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}
