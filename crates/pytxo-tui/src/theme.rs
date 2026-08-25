use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

// Chassis palette — aligned with packages/chroma/tokens.json (source of truth)
pub const VOID: Color = Color::Rgb(20, 23, 28);
pub const CARD: Color = Color::Rgb(30, 35, 43);
pub const TEAL: Color = Color::Rgb(63, 143, 122);
pub const VIOLET: Color = Color::Rgb(224, 106, 58);
pub const GOLD: Color = Color::Rgb(224, 106, 58);
pub const MAGENTA: Color = Color::Rgb(224, 106, 58);
pub const CYAN: Color = Color::Rgb(63, 143, 122);
pub const BORDER: Color = Color::Rgb(61, 68, 80);
pub const MUTED_FG: Color = Color::Rgb(154, 163, 176);
pub const FOREGROUND: Color = Color::Rgb(230, 232, 236);
pub const SELECTION_BG: Color = Color::Rgb(37, 43, 52);

const CHROMA_CYCLE: [Color; 4] = [TEAL, GOLD, TEAL, GOLD];

pub fn border() -> Style {
    Style::default().fg(BORDER)
}

/// Focused panel border — Live accent.
pub fn border_focused() -> Style {
    Style::default().fg(TEAL)
}

/// Splash / welcome cycle only — Live and Cue, not for idle shell panels.
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
        assert_eq!(rgb(VOID), (20, 23, 28));
        assert_eq!(rgb(TEAL), (63, 143, 122));
        assert_eq!(rgb(VIOLET), (224, 106, 58));
        assert_eq!(rgb(GOLD), (224, 106, 58));
        assert_eq!(rgb(MAGENTA), (224, 106, 58));
        assert_eq!(rgb(CYAN), (63, 143, 122));
        assert_eq!(rgb(FOREGROUND), (230, 232, 236));
        assert_eq!(CHROMA_CYCLE[0], TEAL);
        assert_eq!(CHROMA_CYCLE[1], GOLD);
    }
}
