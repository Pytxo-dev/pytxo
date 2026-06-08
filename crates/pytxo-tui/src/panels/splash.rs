use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::theme;

const BANNER: &[&str] = &[
    "██████╗ ██╗   ██╗████████╗██╗  ██╗ ██████╗ ",
    "██╔══██╗╚██╗ ██╔╝╚══██╔══╝╚██╗██╔╝██╔═══██╗",
    "██████╔╝ ╚████╔╝    ██║    ╚███╔╝ ██║   ██║",
    "██╔═══╝   ╚██╔╝     ██║    ██╔██╗ ██║   ██║",
    "██║        ██║      ██║   ██╔╝ ██╗╚██████╔╝",
    "╚═╝        ╚═╝      ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ",
];

pub fn draw(frame: &mut Frame, area: Rect) {
    let mut lines: Vec<Line> = BANNER
        .iter()
        .enumerate()
        .map(|(i, row)| theme::chroma_line(row, i))
        .collect();
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("hypervisor shell", theme::muted()),
        Span::styled(" · ", theme::panel_bg()),
        Span::styled("/help", theme::chroma_cyan()),
    ]));
    lines.push(Line::from(Span::styled(
        "type any command to dismiss",
        theme::muted(),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::chroma_border(0))
        .title(Span::styled(" Scrollback ", theme::chroma_violet()))
        .style(theme::panel_bg());
    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
