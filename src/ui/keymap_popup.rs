use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Clear, Padding, Paragraph},
    Frame,
};

use super::theme::Theme;

const HELP_TEXT: &str = include_str!("../../keymap.txt");

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
            content: Text::from(HELP_TEXT),
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
        self.scroll_y += 1;
    }

    pub fn go_up(&mut self) {
        self.scroll_y = self.scroll_y.saturating_sub(1);
    }

    pub fn go_right(&mut self) {
        self.scroll_x += 1;
    }

    pub fn go_left(&mut self) {
        self.scroll_x = self.scroll_x.saturating_sub(1);
    }

    pub fn draw(&self, frame: &mut Frame<CrosstermBackend<std::io::Stdout>>, theme: &dyn Theme) {
        if !self.visible {
            return;
        }

        let popup_area = Self::get_popup_area(frame.size(), 80, 80);

        let max_scroll = |size: usize, window: u16| {
            let size: u16 = size.try_into().unwrap_or(u16::MAX);
            size.saturating_sub(window)
        };
        let scroll_y = self
            .scroll_y
            .min(max_scroll(self.content.height(), popup_area.height - 2));
        let scroll_x = self
            .scroll_x
            .min(max_scroll(self.content.width(), popup_area.width - 4));

        let paragraph = Paragraph::new(self.content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.search_popup_border())
                    .title("Keybindings")
                    .title_alignment(Alignment::Center)
                    .padding(Padding::horizontal(1)),
            )
            .scroll((scroll_y, scroll_x));

        frame.render_widget(Clear, popup_area);
        frame.render_widget(paragraph, popup_area);
    }

    fn get_popup_area(frame_size: Rect, width_percent: u16, height_percent: u16) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - height_percent) / 2),
                    Constraint::Percentage(height_percent),
                    Constraint::Percentage((100 - height_percent) / 2),
                ]
                .as_ref(),
            )
            .split(frame_size);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - width_percent) / 2),
                    Constraint::Percentage(width_percent),
                    Constraint::Percentage((100 - width_percent) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }
}

impl Default for KeymapPopup {
    fn default() -> Self {
        Self::new()
    }
}
