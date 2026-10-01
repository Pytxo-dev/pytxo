use pytxo_orchestrate::{RunStatusJson, WorkerPane};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::Frame;

use crate::theme;

enum Tone {
    Live,
    Done,
    Failed,
    Queued,
    Settled,
}

fn tone(pane: &WorkerPane) -> Tone {
    match pane.status.as_str() {
        "running" | "starting" | "pending" => Tone::Live,
        "completed" if pane.exit_code == Some(0) => Tone::Done,
        "failed" | "blocked_by_dependency" | "verify_failed" => Tone::Failed,
        "queued" => Tone::Queued,
        _ => Tone::Settled,
    }
}

/// One pane per reviewed task: CLI, state, recent recorded output and owned paths.
pub fn draw(frame: &mut Frame, area: Rect, run: &RunStatusJson, panes: &[WorkerPane]) {
    let waves = panes.iter().map(|pane| pane.wave + 1).max().unwrap_or(0);
    let short_id: String = run.id.chars().take(8).collect();
    let title = run
        .title
        .clone()
        .unwrap_or_else(|| format!("Run {short_id}"));
    let summary = format!(
        " {} · {} {} · {} {} ",
        run.status,
        panes.len(),
        if panes.len() == 1 {
            "worker"
        } else {
            "workers"
        },
        waves,
        if waves == 1 { "wave" } else { "waves" },
    );
    let outer = Block::default()
        .borders(Borders::TOP)
        .border_style(theme::border())
        .title(Span::styled(format!(" {title} "), theme::title()))
        .title_top(Line::from(Span::styled(summary, theme::muted())).right_aligned())
        .style(theme::panel_bg());
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    if panes.is_empty() || inner.height < 3 {
        return;
    }

    let columns = match inner.width {
        150.. => 3,
        90.. => 2,
        _ => 1,
    }
    .min(panes.len());
    let rows = panes.len().div_ceil(columns);
    let row_areas = Layout::vertical(vec![Constraint::Ratio(1, rows as u32); rows]).split(inner);
    for (row, chunk) in panes.chunks(columns).enumerate() {
        let cells = Layout::horizontal(vec![Constraint::Ratio(1, columns as u32); columns])
            .split(row_areas[row]);
        for (pane, cell) in chunk.iter().zip(cells.iter()) {
            draw_pane(frame, *cell, pane);
        }
    }
}

fn draw_pane(frame: &mut Frame, area: Rect, pane: &WorkerPane) {
    let tone = tone(pane);
    let (state, state_style, border) = match tone {
        Tone::Live => (
            "● working".to_string(),
            theme::chroma_cyan(),
            theme::chroma_cyan(),
        ),
        Tone::Done => ("✓ done".to_string(), theme::ok(), theme::border_focused()),
        Tone::Failed => ("✗ failed".to_string(), theme::err(), theme::err()),
        Tone::Queued => ("queued".to_string(), theme::muted(), theme::border()),
        Tone::Settled => (
            pane.status.replace('_', " "),
            theme::muted(),
            theme::border(),
        ),
    };
    let vendor = pane.launcher.as_deref().unwrap_or("Agent");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::horizontal(1))
        .border_style(border)
        .title(Line::from(vec![
            Span::styled(
                format!(" {vendor}"),
                theme::foreground().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" · {} ", pane.task_id), theme::muted()),
        ]))
        .title_top(Line::from(Span::styled(format!(" {state} "), state_style)).right_aligned());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height == 0 {
        return;
    }

    let paths = (!pane.paths.is_empty() && inner.height >= 2).then(|| pane.paths.join(" · "));
    let body_height = inner.height as usize - usize::from(paths.is_some());
    let width = inner.width as usize;
    let mut body: Vec<Line> = if pane.recent.is_empty() {
        let hint = if pane.agent_id.is_some() {
            "No output recorded yet."
        } else {
            "Waits for its inputs."
        };
        vec![Line::from(Span::styled(hint, theme::muted()))]
    } else {
        pane.recent[pane.recent.len().saturating_sub(body_height)..]
            .iter()
            .map(|line| {
                let style = if line.starts_with('✓') {
                    theme::ok()
                } else if line.starts_with('✗') {
                    theme::err()
                } else if line.starts_with("$ ") {
                    theme::accent()
                } else {
                    theme::foreground()
                };
                Line::from(Span::styled(truncate(line, width), style))
            })
            .collect()
    };
    // Output sits at the bottom of the pane, like a terminal.
    while body.len() < body_height {
        body.insert(0, Line::default());
    }
    if let Some(paths) = paths {
        body.push(Line::from(Span::styled(
            truncate(&paths, width),
            theme::muted(),
        )));
    }
    frame.render_widget(Paragraph::new(body).style(Style::default()), inner);
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn pane(task: &str, launcher: &str, status: &str, recent: &[&str]) -> WorkerPane {
        WorkerPane {
            task_id: task.into(),
            wave: 0,
            agent_id: (status != "queued").then(|| format!("agent-{task}")),
            status: status.into(),
            exit_code: (status == "completed").then_some(0),
            launcher: Some(launcher.into()),
            paths: vec![format!("src/{task}.js")],
            recent: recent.iter().map(|line| line.to_string()).collect(),
        }
    }

    #[test]
    fn fleet_shows_each_worker_with_its_cli_state_and_output() {
        let run = RunStatusJson {
            id: "0b7c2f4e-run".into(),
            status: "running".into(),
            repo_root: "/repo".into(),
            started_at: String::new(),
            estimated_tokens_in: None,
            estimated_tokens_out: None,
            estimated_cost_usd: None,
            permission_profile: None,
            isolation_mode: String::new(),
            isolation_backend: String::new(),
            arbitrage_saved_tokens: None,
            wallet_balance_microcredits: None,
            title: Some("Search and dark mode".into()),
            agents: Vec::new(),
        };
        let panes = [
            pane(
                "model",
                "OpenAI Codex",
                "completed",
                &["pass 11 · fail 0", "✓ Task checks passed"],
            ),
            pane(
                "dark-mode",
                "Claude Code",
                "running",
                &["Update(src/style.css)"],
            ),
            pane("readme", "Antigravity", "queued", &[]),
        ];
        let mut terminal = Terminal::new(TestBackend::new(120, 18)).unwrap();
        terminal
            .draw(|frame| draw(frame, frame.area(), &run, &panes))
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        for expected in [
            "Search and dark mode",
            "running · 3 workers · 1 wave",
            "OpenAI Codex · model",
            "✓ done",
            "✓ Task checks passed",
            "Claude Code · dark-mode",
            "● working",
            "Update(src/style.css)",
            "Antigravity · readme",
            "Waits for its inputs.",
            "src/readme.js",
        ] {
            assert!(text.contains(expected), "missing {expected:?}");
        }
    }
}
