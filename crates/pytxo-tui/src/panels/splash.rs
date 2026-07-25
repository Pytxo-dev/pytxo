use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::theme;

/// Minimal welcome strip (no ASCII banner) — keeps cold paint calm and fast.
pub fn draw(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled("Pytxo", theme::title()),
            Span::styled(" · ", theme::muted()),
            Span::styled("hypervisor shell", theme::muted()),
        ]),
        Line::from(Span::styled(
            "/help · Discord: https://discord.gg/AUFRPFjSYv",
            theme::muted(),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chroma_border(0))
        .style(theme::panel_bg());
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}
