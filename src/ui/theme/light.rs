use super::Theme;
use tui::style::Color;

pub struct Light;

impl Theme for Light {
    fn highlight_color(&self) -> Color {
        Color::Rgb(220, 220, 220)
    }

    fn context_viewer_theme(&self) -> &str {
        "base16-ocean.light"
    }
}
