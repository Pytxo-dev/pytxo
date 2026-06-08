use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::theme;

pub struct Scrollback {
    lines: Vec<Line<'static>>,
    max_lines: usize,
}

impl Scrollback {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            max_lines,
        }
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn push(&mut self, text: &str) {
        for line in text.lines() {
            self.lines.push(stylize_line(line));
        }
        if self.lines.len() > self.max_lines {
            let drop = self.lines.len() - self.max_lines;
            self.lines.drain(0..drop);
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, scroll: usize) {
        let visible = area.height.saturating_sub(2) as usize;
        let start = self.lines.len().saturating_sub(visible + scroll);
        let end = (start + visible).min(self.lines.len());
        let body: Vec<Line> = self.lines[start..end].to_vec();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::chroma_border(2))
            .title(ratatui::text::Span::styled(" Scrollback ", theme::chroma_violet()))
            .style(theme::panel_bg());
        frame.render_widget(
            Paragraph::new(body)
                .block(block)
                .wrap(Wrap { trim: false })
                .style(theme::muted()),
            area,
        );
    }
}

fn stylize_line(line: &str) -> Line<'static> {
    let muted = theme::muted();
    if !line.trim_start().starts_with('{') && !line.contains("\"waves\"") {
        return Line::from(Span::styled(line.to_string(), muted));
    }
    let key_style = theme::chroma_cyan();
    let val_style = theme::chroma_gold();
    let bytes = line.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            let quoted = &line[start..i.min(line.len())];
            let tail = line[i..].trim_start();
            let style = if tail.starts_with(':') {
                key_style
            } else {
                val_style
            };
            spans.push(Span::styled(quoted.to_string(), style));
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            spans.push(Span::styled(line[start..i].to_string(), muted));
        }
    }
    if spans.is_empty() {
        Line::from(Span::styled(line.to_string(), muted))
    } else {
        Line::from(spans)
    }
}
