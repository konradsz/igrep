use super::Theme;
use tui::style::{Color, Style};

pub struct Light;

impl Theme for Light {
    fn highlight_color(&self) -> Style {
        Style::default().bg(Color::Rgb(220, 220, 220))
    }

    fn context_highlight_color(&self) -> Color {
        Color::Rgb(23, 30, 102)
    }

    fn context_highlight_theme(&self) -> &str {
        "base16-ocean.light"
    }
}
