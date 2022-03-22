use super::Theme;
use tui::style::{Color, Style};

pub struct Dark;

impl Theme for Dark {
    fn highlight_color(&self) -> Style {
        Style::default().bg(Color::Rgb(58, 58, 58))
    }

    fn bottom_bar_color(&self) -> Color {
        Color::Rgb(58, 58, 58)
    }

    fn bottom_bar_font_color(&self) -> Color {
        Color::Rgb(147, 147, 147)
    }
}
