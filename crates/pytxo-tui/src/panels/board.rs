use pytxo_core::AgentSpec;
use pytxo_orchestrate::{DashboardSnapshot, DoctorReport};
use pytxo_runner::HitlRequest;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap};
use ratatui::Frame;

use crate::theme;

pub struct BoardView {
    pub run_selected: usize,
    pub hitl_selected: usize,
}

// The board renderer receives independent live model slices to avoid allocating a
// duplicate presentation model on every terminal frame.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    doctor: &DoctorReport,
    active_run: Option<&str>,
    trust_tier: &str,
    org_policy: Option<&str>,
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

    draw_header(
        frame, chunks[0], snapshot, doctor, active_run, trust_tier, org_policy, agents,
    );
    draw_runs(frame, chunks[1], snapshot, view.run_selected);
    draw_hitl(frame, chunks[2], snapshot, view.hitl_selected);
}

#[allow(clippy::too_many_arguments)]
fn draw_header(
    frame: &mut Frame,
    area: Rect,
    snapshot: &DashboardSnapshot,
    doctor: &DoctorReport,
    active_run: Option<&str>,
    trust_tier: &str,
    org_policy: Option<&str>,
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
    let active = active_run.map_or_else(
        || "—".to_string(),
        |id| {
            let run = snapshot.runs.iter().find(|run| run.id == id);
            run.and_then(|run| run.title.clone())
                .unwrap_or_else(|| id.chars().take(8).collect())
        },
    );
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
    let total_active: usize = snapshot.domains.iter().map(|d| d.active_runs).sum();

    // Line 1: identity (product, version, repo, trust)
    let mut identity = vec![
        Span::styled(" pytxo ", theme::title()),
        Span::styled(format!("v{} ", snapshot.version), theme::chroma_cyan()),
        Span::styled(truncate(&snapshot.repo_root, 36), theme::foreground()),
        Span::styled(format!(" · {trust_tier}"), theme::muted()),
    ];
    if let Some(org) = org_policy {
        identity.push(Span::styled(format!(" · {org}"), theme::chroma_cyan()));
    }
    frame.render_widget(
        Paragraph::new(Line::from(identity)).style(theme::header_bg()),
        Rect { height: 1, ..area },
    );

    // Line 2: status (doctor, run, agents, domains)
    if area.height >= 2 {
        let status = Line::from(vec![
            Span::styled(" ", theme::muted()),
            Span::styled(doctor_label, doctor_style),
            Span::styled(format!(" · run {active} · {agent_hint}"), theme::muted()),
            Span::styled(
                format!(
                    " · {} domain(s) · {} active",
                    snapshot.domains.len(),
                    total_active
                ),
                theme::muted(),
            ),
        ]);
        let status_area = Rect {
            y: area.y + 1,
            height: 1,
            ..area
        };
        if snapshot.domains.len() > 1 {
            draw_domains_strip(frame, status_area, snapshot);
        } else {
            frame.render_widget(
                Paragraph::new(status).style(theme::header_bg()),
                status_area,
            );
        }
    }
}

fn draw_domains_strip(frame: &mut Frame, strip: Rect, snapshot: &DashboardSnapshot) {
    let parts: Vec<String> = snapshot
        .domains
        .iter()
        .take(4)
        .map(|d| {
            let name = std::path::Path::new(&d.repo_root)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| truncate(&d.repo_root, 12));
            let st = d.latest_run_status.as_deref().unwrap_or("—");
            let active = d.active_runs;
            format!("{name}:{active}/{st}")
        })
        .collect();
    let more = if snapshot.domains.len() > 4 {
        format!(" +{}", snapshot.domains.len() - 4)
    } else {
        String::new()
    };
    frame.render_widget(
        Paragraph::new(format!(" domains {}{}", parts.join(" · "), more)).style(theme::muted()),
        strip,
    );
}

fn draw_runs(frame: &mut Frame, area: Rect, snapshot: &DashboardSnapshot, selected: usize) {
    let header = Row::new(vec!["Request", "Status", "Project", "Workers", "Cost"])
        .style(theme::chroma_violet())
        .bottom_margin(1);
    let rows: Vec<Row> = snapshot
        .runs
        .iter()
        .map(|r| {
            let repo_short = std::path::Path::new(&r.repo_root)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| truncate(&r.repo_root, 10));
            let short_id: String = r.id.chars().take(8).collect();
            Row::new(vec![
                Cell::from(r.title.clone().unwrap_or_else(|| format!("Run {short_id}"))),
                Cell::from(r.status.clone()),
                Cell::from(repo_short),
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
            ratatui::layout::Constraint::Min(24),
            ratatui::layout::Constraint::Length(12),
            ratatui::layout::Constraint::Length(16),
            ratatui::layout::Constraint::Length(8),
            ratatui::layout::Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(theme::border())
            .title(Span::styled(" runs ", theme::title()))
            .style(theme::panel_bg()),
    )
    .row_highlight_style(theme::selection_bg().add_modifier(Modifier::BOLD));
    let mut state = TableState::default().with_selected(Some(selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_hitl(frame: &mut Frame, area: Rect, snapshot: &DashboardSnapshot, selected: usize) {
    let pending = &snapshot.hitl_pending;
    let body = if pending.is_empty() {
        vec![Line::from(vec![
            Span::styled("none pending", theme::muted()),
            Span::styled(" · Tab · Ctrl+A · Ctrl+X when waiting", theme::muted()),
        ])]
    } else {
        let mut lines: Vec<Line> = pending
            .iter()
            .enumerate()
            .map(|(i, r)| hitl_line(r, i == selected))
            .collect();
        if lines.len() == 1 {
            lines.push(Line::from(Span::styled(
                " Tab cycle · Ctrl+A approve · Ctrl+X deny",
                theme::muted(),
            )));
        }
        lines
    };
    let border = if pending.is_empty() {
        theme::border()
    } else {
        theme::border_focused()
    };
    frame.render_widget(
        Paragraph::new(body)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(border)
                    .title(Span::styled(
                        " Approvals ",
                        if pending.is_empty() {
                            theme::muted()
                        } else {
                            theme::chroma_magenta()
                        },
                    ))
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
