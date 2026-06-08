use pytxo_core::AgentSpec;
use pytxo_orchestrate::{DashboardSnapshot, DoctorReport};
use pytxo_runner::HitlRequest;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap};
use ratatui::Frame;

use crate::theme;

pub struct BoardView {
    pub run_selected: usize,
    pub hitl_selected: usize,
}

pub fn draw(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    doctor: &DoctorReport,
    active_run: Option<&str>,
    trust_tier: &str,
    agents: &[AgentSpec],
    view: &BoardView,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(2),
            ratatui::layout::Constraint::Min(4),
            ratatui::layout::Constraint::Length(3),
        ])
        .split(area);

    draw_header(frame, chunks[0], snapshot, doctor, active_run, trust_tier, agents);
    draw_runs(frame, chunks[1], snapshot, view.run_selected);
    draw_hitl(frame, chunks[2], snapshot, view.hitl_selected);
}

fn draw_header(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    doctor: &DoctorReport,
    active_run: Option<&str>,
    trust_tier: &str,
    agents: &[AgentSpec],
) {
    let doctor_unknown = doctor.checks.is_empty();
    let doctor_ok = !doctor_unknown && doctor.all_ok();
    let doctor_label = if doctor_unknown {
        "● …"
    } else if doctor_ok {
        "● ok"
    } else {
        "● fail"
    };
    let doctor_style = if doctor_unknown {
        theme::muted()
    } else if doctor_ok {
        theme::ok()
    } else {
        theme::err()
    };
    let active = active_run.unwrap_or("—");
    let agent_hint = if agents.is_empty() {
        "generic".to_string()
    } else {
        agents
            .iter()
            .map(|a| {
                a.cli_adapter
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| a.name.clone())
            })
            .collect::<Vec<_>>()
            .join(",")
    };
    let text = Line::from(vec![
        Span::styled(" pytxo ", theme::title()),
        Span::styled(format!("v{} ", snapshot.version), theme::chroma_cyan()),
        Span::styled(
            truncate(&snapshot.repo_root, 28),
            Style::default().fg(ratatui::style::Color::White),
        ),
        Span::styled(format!(" · {trust_tier} · "), theme::muted()),
        Span::styled(doctor_label, doctor_style),
        Span::styled(format!(" · run {active} · {agent_hint}"), theme::muted()),
        Span::styled(
            format!(" · {} domain(s)", snapshot.domains.len()),
            theme::muted(),
        ),
    ]);
    frame.render_widget(Paragraph::new(text).style(theme::header_bg()), area);
}

fn draw_runs(frame: &mut Frame, area: Rect, snapshot: &DashboardSnapshot, selected: usize) {
    let header = Row::new(vec!["Run", "Status", "Agents", "Cost"])
        .style(theme::chroma_violet())
        .bottom_margin(1);
    let rows: Vec<Row> = snapshot
        .runs
        .iter()
        .map(|r| {
            Row::new(vec![
                Cell::from(truncate(&r.id, 24)),
                Cell::from(r.status.clone()),
                Cell::from(r.agents.len().to_string()),
                Cell::from(
                    r.estimated_cost_usd
                        .map(|c| format!("{c:.4}"))
                        .unwrap_or_else(|| "—".into()),
                )
                .style(
                    r.estimated_cost_usd
                        .map(|_| theme::warn())
                        .unwrap_or(theme::muted()),
                ),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Percentage(40),
            ratatui::layout::Constraint::Length(12),
            ratatui::layout::Constraint::Length(8),
            ratatui::layout::Constraint::Length(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(theme::chroma_border(3))
            .title(Span::styled(" board ", theme::chroma_gold()))
            .style(theme::panel_bg()),
    )
    .row_highlight_style(
        Style::default()
            .bg(ratatui::style::Color::Rgb(30, 20, 46))
            .add_modifier(Modifier::BOLD),
    );
    let mut state = TableState::default().with_selected(Some(selected));
    frame.render_stateful_widget(table, area, &mut state);

    if let Some(run) = snapshot.runs.get(selected) {
        if !run.agents.is_empty() && area.height > 5 {
            let waves: String = run
                .agents
                .iter()
                .map(|a| format!("w{}:{}", a.wave, a.id))
                .collect::<Vec<_>>()
                .join(" ");
            let detail_area = Rect {
                y: area.y + area.height.saturating_sub(2),
                height: 1,
                ..area
            };
            frame.render_widget(
                Paragraph::new(truncate(&waves, area.width as usize)).style(theme::muted()),
                detail_area,
            );
        }
    }
}

fn draw_hitl(frame: &mut Frame, area: Rect, snapshot: &DashboardSnapshot, selected: usize) {
    let pending = &snapshot.hitl_pending;
    let body = if pending.is_empty() {
        vec![Line::from(vec![
            Span::styled("hitl", theme::chroma_magenta()),
            Span::styled(" — none pending", theme::muted()),
        ])]
    } else {
        pending
            .iter()
            .enumerate()
            .map(|(i, r)| hitl_line(r, i == selected))
            .collect()
    };
    frame.render_widget(
        Paragraph::new(body)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(if pending.is_empty() {
                        theme::border()
                    } else {
                        theme::chroma_border(2)
                    })
                    .style(theme::panel_bg()),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn hitl_line(req: &HitlRequest, selected: bool) -> Line<'static> {
    let style = if selected {
        theme::accent().add_modifier(Modifier::BOLD)
    } else {
        theme::muted()
    };
    Line::from(Span::styled(
        format!(" {} [{}] {}", req.id, req.agent_key, req.action),
        style,
    ))
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}
