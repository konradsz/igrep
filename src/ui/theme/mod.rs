pub mod dark;
pub mod light;

use clap::ArgEnum;
use strum_macros::Display;
use tui::style::{Color, Modifier, Style};

#[derive(Display, Copy, Clone, Debug, ArgEnum)]
#[strum(serialize_all = "lowercase")]
pub enum ThemeVariant {
    Light,
    Dark,
}

pub trait Theme {
    // Matches list styles
    fn background_color(&self) -> Style {
        Style::default()
    }

    fn list_font_color(&self) -> Style {
        Style::default()
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

    fn highlight_color(&self) -> Color;

    // Context viewer styles
    fn context_viewer_theme(&self) -> &str;

    // Bottom bar styles
    fn bottom_bar_color(&self) -> Color {
        Color::Reset
    }

    fn bottom_bar_font_color(&self) -> Color {
        Color::Reset
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

    fn error_state_style(&self) -> Style {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Red)
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
