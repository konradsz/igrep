use super::Theme;
use tui::style::{Color, Modifier, Style};

pub struct Dark;

impl Theme for Dark {
    fn background_color(&self) -> Style {
        Style::default()
    }

    fn list_font_color(&self) -> Style {
        Style::default()
    }

    fn highlight_color(&self) -> Style {
        Style::default().bg(Color::Rgb(58, 58, 58))
    }

    fn file_path_color(&self) -> Style {
        Style::default().fg(Color::LightMagenta)
    }

    fn line_number_color(&self) -> Style {
        Style::default().fg(Color::Green)
    }

    fn match_color(&self) -> Style {
        Style::default().fg(Color::Red)
    }

    fn bottom_bar_color(&self) -> Color {
        Color::Rgb(58, 58, 58)
    }

    fn bottom_bar_font_color(&self) -> Color {
        Color::Rgb(147, 147, 147)
    }

    fn bottom_bar_style(&self) -> Style {
        Style::default()
            .bg(self.bottom_bar_color())
            .fg(self.bottom_bar_font_color())
    }

    fn searching_state_style(&self) -> Style {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::LightRed)
            .fg(Color::Black)
    }

    fn finished_state_style(&self) -> Style {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Green)
            .fg(Color::Black)
    }

    fn invalid_input_color(&self) -> Color {
        Color::Red
    }
}
