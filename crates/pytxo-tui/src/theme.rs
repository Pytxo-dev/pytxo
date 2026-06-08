use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

// Chroma palette — aligned with apps/web/src/app/globals.css (dark)
pub const VOID: Color = Color::Rgb(2, 2, 5);
pub const CARD: Color = Color::Rgb(10, 10, 15);
pub const TEAL: Color = Color::Rgb(45, 212, 191);
pub const VIOLET: Color = Color::Rgb(167, 139, 250);
pub const GOLD: Color = Color::Rgb(251, 191, 36);
pub const MAGENTA: Color = Color::Rgb(232, 121, 249);
pub const CYAN: Color = Color::Rgb(34, 211, 238);
pub const BORDER: Color = Color::Rgb(39, 39, 42);
pub const MUTED_FG: Color = Color::Rgb(148, 148, 168);

const CHROMA_CYCLE: [Color; 4] = [MAGENTA, GOLD, CYAN, VIOLET];

pub fn border() -> Style {
    Style::default().fg(BORDER)
}

pub fn chroma_border(idx: usize) -> Style {
    Style::default().fg(CHROMA_CYCLE[idx % CHROMA_CYCLE.len()])
}

pub fn accent() -> Style {
    Style::default().fg(TEAL)
}

pub fn chroma_magenta() -> Style {
    Style::default().fg(MAGENTA)
}

pub fn chroma_gold() -> Style {
    Style::default().fg(GOLD)
}

pub fn chroma_cyan() -> Style {
    Style::default().fg(CYAN)
}

pub fn chroma_violet() -> Style {
    Style::default().fg(VIOLET)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED_FG)
}

pub fn title() -> Style {
    Style::default()
        .fg(VIOLET)
        .add_modifier(Modifier::BOLD)
}

pub fn panel_bg() -> Style {
    Style::default().bg(CARD)
}

pub fn header_bg() -> Style {
    Style::default().bg(VOID)
}

pub fn ok() -> Style {
    Style::default().fg(Color::Rgb(74, 222, 128))
}

pub fn err() -> Style {
    Style::default().fg(Color::Rgb(248, 113, 113))
}

pub fn warn() -> Style {
    chroma_gold()
}

/// Per-line chroma shift for ASCII banner rows.
pub fn chroma_line(text: &str, line_idx: usize) -> Line<'static> {
    let color = CHROMA_CYCLE[line_idx % CHROMA_CYCLE.len()];
    Line::from(Span::styled(text.to_string(), Style::default().fg(color)))
}

#[allow(dead_code)]
pub fn chroma_title_spans(text: &str) -> Vec<Span<'static>> {
    text.chars()
        .enumerate()
        .map(|(i, ch)| {
            Span::styled(
                ch.to_string(),
                Style::default()
                    .fg(CHROMA_CYCLE[i % CHROMA_CYCLE.len()])
                    .add_modifier(Modifier::BOLD),
            )
        })
        .collect()
}
