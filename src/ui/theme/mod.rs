pub mod dark;

use tui::style::{Color, Style};

pub trait Theme {
    // Matches list styles
    fn background_color(&self) -> Style;
    fn list_font_color(&self) -> Style;
    fn highlight_color(&self) -> Style;
    fn file_path_color(&self) -> Style;
    fn line_number_color(&self) -> Style;
    fn match_color(&self) -> Style;

    // Bottom bar styles
    fn bottom_bar_color(&self) -> Color;
    fn bottom_bar_font_color(&self) -> Color;
    fn bottom_bar_style(&self) -> Style;
    fn searching_state_style(&self) -> Style;
    fn finished_state_style(&self) -> Style;
    fn invalid_input_color(&self) -> Color;
}
