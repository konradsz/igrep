use super::Theme;
use tui::style::{Color, Style};

pub struct Light;

impl Theme for Light {
    fn highlight_color(&self) -> Style {
        Style::default().bg(Color::Rgb(220, 220, 220))
    }
}
