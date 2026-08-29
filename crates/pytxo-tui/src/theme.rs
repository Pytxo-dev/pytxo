use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

// Chroma ribbon palette — aligned with packages/chroma/tokens.json (source of truth)
pub const VOID: Color = Color::Rgb(5, 5, 7);
pub const CARD: Color = Color::Rgb(17, 17, 19);
pub const TEAL: Color = Color::Rgb(123, 224, 122);
pub const VIOLET: Color = Color::Rgb(196, 75, 255);
pub const MAGENTA: Color = Color::Rgb(255, 75, 154);
pub const ORANGE: Color = Color::Rgb(255, 106, 61);
pub const GOLD: Color = Color::Rgb(245, 197, 66);
pub const CYAN: Color = Color::Rgb(62, 224, 208);
pub const BORDER: Color = Color::Rgb(42, 42, 46);
pub const MUTED_FG: Color = Color::Rgb(161, 161, 170);
pub const FOREGROUND: Color = Color::Rgb(237, 237, 239);
pub const SELECTION_BG: Color = Color::Rgb(24, 24, 27);

const CHROMA_CYCLE: [Color; 6] = [VIOLET, MAGENTA, ORANGE, GOLD, TEAL, CYAN];

pub fn border() -> Style {
    Style::default().fg(BORDER)
}

/// Focused panel border — Live accent.
pub fn border_focused() -> Style {
    Style::default().fg(TEAL)
}

/// Splash / welcome cycle only — logo spectrum, not for idle shell panels.
pub fn chroma_border(idx: usize) -> Style {
    Style::default().fg(CHROMA_CYCLE[idx % CHROMA_CYCLE.len()])
}

pub fn accent() -> Style {
    Style::default().fg(TEAL)
}

pub fn foreground() -> Style {
    Style::default().fg(FOREGROUND)
}

pub fn selection_bg() -> Style {
    Style::default().bg(SELECTION_BG)
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
    Style::default().fg(FOREGROUND).add_modifier(Modifier::BOLD)
}

pub fn panel_bg() -> Style {
    Style::default().bg(CARD)
}

pub fn header_bg() -> Style {
    Style::default().bg(VOID)
}

pub fn ok() -> Style {
    Style::default().fg(TEAL)
}

pub fn err() -> Style {
    Style::default().fg(Color::Rgb(217, 107, 107))
}

pub fn warn() -> Style {
    chroma_gold()
}

/// Per-line cycle for ASCII banner rows (splash only).
#[allow(dead_code)] // Used by richer splash variants; keep for chroma demos.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(c: Color) -> (u8, u8, u8) {
        match c {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => panic!("expected rgb"),
        }
    }

    #[test]
    fn chroma_palette_matches_tokens_json() {
        assert_eq!(rgb(VOID), (5, 5, 7));
        assert_eq!(rgb(TEAL), (123, 224, 122));
        assert_eq!(rgb(VIOLET), (196, 75, 255));
        assert_eq!(rgb(MAGENTA), (255, 75, 154));
        assert_eq!(rgb(ORANGE), (255, 106, 61));
        assert_eq!(rgb(GOLD), (245, 197, 66));
        assert_eq!(rgb(CYAN), (62, 224, 208));
        assert_eq!(rgb(FOREGROUND), (237, 237, 239));
        assert_eq!(CHROMA_CYCLE[0], VIOLET);
        assert_eq!(CHROMA_CYCLE[1], MAGENTA);
        assert_eq!(CHROMA_CYCLE.len(), 6);
    }
}
