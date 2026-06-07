use pytxo_core::PermissionProfile;
use pytxo_orchestrate::trust_repo;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::theme;

const TIERS: [(PermissionProfile, &str, &str); 4] = [
    (
        PermissionProfile::DeepSpace,
        "Deep Space",
        "Air-gapped bubble — minimal host access",
    ),
    (
        PermissionProfile::Orbit,
        "Orbit",
        "Default engineering — approve-to-flush (recommended)",
    ),
    (
        PermissionProfile::Galaxy,
        "Galaxy",
        "Host tools + HITL for high-risk actions",
    ),
    (
        PermissionProfile::Supernova,
        "Supernova",
        "Full host privileges — use with care",
    ),
];

pub struct TrustModal {
    pub selected: usize,
    pub repo_path: String,
}

impl TrustModal {
    pub fn new(repo_path: String) -> Self {
        Self {
            selected: 1,
            repo_path,
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < TIERS.len() {
            self.selected += 1;
        }
    }

    pub fn accept(&self, repo: &std::path::Path) -> anyhow::Result<PermissionProfile> {
        let (profile, _, _) = TIERS[self.selected];
        trust_repo(repo, profile)?;
        Ok(profile)
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);
        let popup = centered_rect(72, 70, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::accent())
            .title(Span::styled(" Trust ", theme::title()))
            .style(theme::panel_bg());
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(inner);

        let header = vec![
            Line::from(Span::styled(
                "Do you trust this folder?",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(&self.repo_path, theme::muted())),
        ];
        frame.render_widget(Paragraph::new(header), chunks[0]);

        let tier_lines: Vec<Line> = TIERS
            .iter()
            .enumerate()
            .map(|(i, (_, name, desc))| {
                let mark = if i == self.selected { "›" } else { " " };
                let style = if i == self.selected {
                    theme::accent().add_modifier(Modifier::BOLD)
                } else {
                    theme::muted()
                };
                Line::from(vec![
                    Span::styled(format!("{mark} {name}"), style),
                    Span::styled(format!(" — {desc}"), theme::muted()),
                ])
            })
            .collect();
        frame.render_widget(
            Paragraph::new(tier_lines).wrap(Wrap { trim: true }),
            chunks[1],
        );

        frame.render_widget(
            Paragraph::new("↑/↓ select · Enter trust · q quit without trusting")
                .style(theme::muted()),
            chunks[2],
        );
    }
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
