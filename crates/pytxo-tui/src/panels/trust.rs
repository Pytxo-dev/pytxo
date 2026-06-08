use pytxo_core::PermissionProfile;
use pytxo_orchestrate::trust_repo;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::theme;

const TIERS: [(PermissionProfile, &str, &str, fn() -> Style); 4] = [
    (
        PermissionProfile::DeepSpace,
        "Deep Space",
        "Air-gapped bubble — minimal host access",
        theme::chroma_cyan,
    ),
    (
        PermissionProfile::Orbit,
        "Orbit",
        "Default engineering — approve-to-flush (recommended)",
        theme::accent,
    ),
    (
        PermissionProfile::Galaxy,
        "Galaxy",
        "Host tools + HITL for high-risk actions",
        theme::chroma_violet,
    ),
    (
        PermissionProfile::Supernova,
        "Supernova",
        "Full host privileges — use with care",
        theme::chroma_gold,
    ),
];

pub struct TrustModal {
    pub selected: usize,
    pub folders: Vec<String>,
}

impl TrustModal {
    pub fn new(folders: Vec<String>) -> Self {
        Self {
            selected: 1,
            folders,
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
        let (profile, _, _, _) = TIERS[self.selected];
        trust_repo(repo, profile)?;
        Ok(profile)
    }

    fn heading(&self) -> String {
        if self.folders.len() > 1 {
            "Do you trust these folders for Pytxo agents?".into()
        } else {
            "Do you trust this folder for Pytxo agents?".into()
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);
        let popup = centered_rect(74, 72, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::chroma_border(0))
            .title(Span::styled(" Workspace trust ", theme::title()))
            .style(theme::panel_bg());
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(3),
                Constraint::Length(2),
                Constraint::Min(8),
                Constraint::Length(2),
            ])
            .split(inner);

        let mut header = vec![
            Line::from(Span::styled(
                self.heading(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];
        for path in &self.folders {
            header.push(Line::from(Span::styled(
                format!("  • {path}"),
                theme::chroma_cyan(),
            )));
        }
        frame.render_widget(Paragraph::new(header), chunks[0]);

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "Agents may read, edit, and run commands in trusted paths.",
                    theme::muted(),
                )),
                Line::from(Span::styled(
                    "Choose a permission tier:",
                    theme::muted(),
                )),
            ]),
            chunks[1],
        );

        let tier_lines: Vec<Line> = TIERS
            .iter()
            .enumerate()
            .map(|(i, (_, name, desc, style_fn))| {
                let mark = if i == self.selected { "›" } else { " " };
                let tier_style = if i == self.selected {
                    style_fn().add_modifier(Modifier::BOLD)
                } else {
                    theme::muted()
                };
                Line::from(vec![
                    Span::styled(format!("{mark} {name}"), tier_style),
                    Span::styled(format!(" — {desc}"), theme::muted()),
                ])
            })
            .collect();
        frame.render_widget(
            Paragraph::new(tier_lines).wrap(Wrap { trim: true }),
            chunks[3],
        );

        frame.render_widget(
            Paragraph::new("↑/↓ select tier · Enter trust folder · Esc decline")
                .style(theme::chroma_magenta()),
            chunks[4],
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
