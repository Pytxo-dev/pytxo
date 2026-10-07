use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::Frame;

use crate::theme;

/// First-paint quick start (no ASCII banner): what to type, what happens next.
pub fn draw(frame: &mut Frame, area: Rect) {
    let step = |n: &'static str, text: &'static str| {
        Line::from(vec![
            Span::styled(n, theme::chroma_cyan()),
            Span::styled(text, theme::foreground()),
        ])
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("Pytxo", theme::title()),
            Span::styled(" · many coding agents, one verified change", theme::muted()),
        ]),
        Line::from(""),
        step("1  ", "Type what you want changed, in plain words."),
        Line::from(Span::styled(
            "   e.g. fix src/parser.rs so empty input returns an error; add a test",
            theme::muted(),
        )),
        step(
            "2  ",
            "Check the plan, then /run. Each task gets its own copy of the project.",
        ),
        step("3  ", "Review and Apply the result in Pytxo Desktop."),
        Line::from(""),
        Line::from(Span::styled(
            "/help commands · /agents your CLIs · Ctrl+Q quit",
            theme::muted(),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chroma_border(0))
        .padding(Padding::new(2, 2, 1, 0))
        .style(theme::panel_bg());
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}
