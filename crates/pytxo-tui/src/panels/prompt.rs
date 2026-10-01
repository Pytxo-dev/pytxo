use ratatui::layout::Rect;
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
        let idx = self
            .history_idx
            .unwrap_or(self.history.len())
            .saturating_sub(1);
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

    /// Context-sensitive footer: shell idle, HITL pending, or custom status.
    pub fn draw(
        &self,
        frame: &mut Frame,
        area: Rect,
        status: &str,
        hitl_pending: bool,
        trust_phase: bool,
    ) {
        let prompt = Line::from(vec![
            Span::styled("pytxo> ", theme::accent()),
            Span::styled(&self.buffer, theme::foreground()),
            Span::styled("▌", theme::chroma_magenta()),
        ]);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_focused())
            .title(Span::styled(" Prompt ", theme::title()))
            .style(theme::panel_bg());
        let footer = if !status.is_empty() {
            status.to_string()
        } else if trust_phase {
            "↑↓ select · Enter trust · q quit".to_string()
        } else if hitl_pending {
            "Tab cycle · Ctrl+A approve · Ctrl+X deny · Ctrl+O fleet/output".to_string()
        } else {
            "Enter submit · PgUp/PgDn run · Ctrl+O fleet/output · /help · Ctrl+Q quit".to_string()
        };
        let inner = block.inner(area);
        frame.render_widget(Paragraph::new(prompt).block(block), area);
        if inner.height > 1 {
            let hint_area = Rect {
                y: inner.y + inner.height.saturating_sub(1),
                height: 1,
                ..inner
            };
            frame.render_widget(Paragraph::new(footer).style(theme::muted()), hint_area);
        }
    }
}
