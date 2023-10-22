use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    text::Text,
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame,
};

use super::theme::Theme;

include!(concat!(env!("OUT_DIR"), "/keybindings.rs"));

pub struct KeymapPopup {
    visible: bool,
    scroll_y: u16,
    scroll_x: u16,
    content: Text<'static>,
}

impl KeymapPopup {
    pub fn new() -> Self {
        Self {
            visible: false,
            scroll_y: 0,
            scroll_x: 0,
            content: Text::from(KEYBINDINGS_TABLE),
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.scroll_y = 0;
            self.scroll_x = 0;
        }
    }

    pub fn go_down(&mut self) {
        self.scroll_y = self.scroll_y.saturating_add(1).min(KEYBINDINGS_LEN);
    }

    pub fn go_up(&mut self) {
        self.scroll_y = self.scroll_y.saturating_sub(1);
    }

    pub fn go_right(&mut self) {
        self.scroll_x = self.scroll_x.saturating_add(1).min(KEYBINDINGS_LINE_LEN);
    }

    pub fn go_left(&mut self) {
        self.scroll_x = self.scroll_x.saturating_sub(1);
    }

    pub fn draw(&self, frame: &mut Frame<CrosstermBackend<std::io::Stdout>>, theme: &dyn Theme) {
        if !self.visible {
            return;
        }

        let popup_area = Self::get_popup_area(frame.size());

        let max_y = KEYBINDINGS_LEN.saturating_sub(popup_area.height - 4);
        let scroll_y = self.scroll_y.min(max_y);
        let max_x = KEYBINDINGS_LINE_LEN.saturating_sub(popup_area.width - 4);
        let scroll_x = self.scroll_x.min(max_x);

        let paragraph = Paragraph::new(self.content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.search_popup_border())
                    .title(concat!(
                        " ",
                        env!("CARGO_PKG_NAME"),
                        " ",
                        env!("CARGO_PKG_VERSION"),
                        " "
                    ))
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1)),
            )
            .scroll((scroll_y, scroll_x));

        frame.render_widget(Clear, popup_area);
        frame.render_widget(paragraph, popup_area);
    }

    fn get_popup_area(frame_size: Rect) -> Rect {
        let height = (KEYBINDINGS_LEN + 4).min((frame_size.height as f64 * 0.8) as u16);
        let y = (frame_size.height - height) / 2;

        let width = (KEYBINDINGS_LINE_LEN + 4).min((frame_size.width as f64 * 0.8) as u16);
        let x = (frame_size.width - width) / 2;

        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl Default for KeymapPopup {
    fn default() -> Self {
        Self::new()
    }
}
