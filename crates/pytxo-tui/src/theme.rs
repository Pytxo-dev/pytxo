use ratatui::style::{Color, Modifier, Style};

pub fn border() -> Style {
    Style::default().fg(Color::Rgb(39, 39, 42))
}

pub fn accent() -> Style {
    Style::default().fg(Color::Rgb(45, 212, 191))
}

pub fn muted() -> Style {
    Style::default().fg(Color::Rgb(148, 148, 168))
}

pub fn title() -> Style {
    Style::default()
        .fg(Color::Rgb(167, 139, 250))
        .add_modifier(Modifier::BOLD)
}

pub fn panel_bg() -> Style {
    Style::default().bg(Color::Rgb(10, 10, 15))
}

pub fn header_bg() -> Style {
    Style::default().bg(Color::Rgb(2, 2, 5))
}

pub fn ok() -> Style {
    Style::default().fg(Color::Rgb(74, 222, 128))
}

pub fn err() -> Style {
    Style::default().fg(Color::Rgb(248, 113, 113))
}
