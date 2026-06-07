use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::theme;

pub struct Prompt {
    pub buffer: String,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            history: Vec::new(),
            history_idx: None,
        }
    }

    pub fn push_history(&mut self, line: String) {
        if line.trim().is_empty() {
            return;
        }
        if self.history.last() != Some(&line) {
            self.history.push(line);
        }
        self.history_idx = None;
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let idx = self.history_idx.unwrap_or(self.history.len()).saturating_sub(1);
        self.history_idx = Some(idx);
        self.buffer = self.history[idx].clone();
    }

    pub fn history_down(&mut self) {
        let Some(idx) = self.history_idx else {
            return;
        };
        if idx + 1 >= self.history.len() {
            self.history_idx = None;
            self.buffer.clear();
        } else {
            let next = idx + 1;
            self.history_idx = Some(next);
            self.buffer = self.history[next].clone();
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect, status: &str) {
        let prompt = Line::from(vec![
            Span::styled("pytxo> ", theme::accent()),
            Span::styled(&self.buffer, Style::default().fg(ratatui::style::Color::White)),
            Span::styled("▌", theme::accent()),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border())
            .title(Span::styled(" Prompt ", theme::accent()))
            .style(theme::panel_bg());
        let footer = if status.is_empty() {
            "Enter submit · Ctrl+C /q quit · /help commands".to_string()
        } else {
            status.to_string()
        };
        let inner = block.inner(area);
        frame.render_widget(Paragraph::new(prompt).block(block), area);
        if inner.height > 1 {
            let hint_area = Rect {
                y: inner.y + inner.height.saturating_sub(1),
                height: 1,
                ..inner
            };
            frame.render_widget(
                Paragraph::new(footer).style(theme::muted()),
                hint_area,
            );
        }
    }
}
