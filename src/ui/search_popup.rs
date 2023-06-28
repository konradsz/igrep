use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct SearchPopup {
    visible: bool,
    pattern: String,
    edited_pattern: String,
}

/* TODO:
- allow for input
- allow for backspace
- allow for a delete key
- handle arrow keys
- ENTER to search new pattern
- ESC to cancel search, old pattern should stay
- handle long input
*/

impl SearchPopup {
    pub fn new(pattern: String) -> Self {
        Self {
            visible: false,
            pattern: pattern.clone(),
            edited_pattern: pattern,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_pattern(&mut self, pattern: String) {
        self.pattern = pattern.clone();
        self.edited_pattern = pattern;
    }

    pub fn reset_edited_pattern(&mut self) {
        self.edited_pattern = self.pattern.clone();
    }

    pub fn insert_char(&mut self, c: char) {
        self.edited_pattern.push(c);
    }

    pub fn remove_char(&mut self) {
        self.edited_pattern.pop();
    }

    pub fn draw(&self, frame: &mut Frame<CrosstermBackend<std::io::Stdout>>) {
        if !self.visible {
            return;
        }

        let block = Block::default()
            .title("Pattern")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        let popup_area = Self::get_popup_area(frame.size(), 10);
        frame.render_widget(Clear, popup_area);
        frame.render_widget(block, popup_area);

        let mut text_area = popup_area.clone();
        text_area.y += 1; // one line below the border
        text_area.x += 2; // two chars to the right

        let max_text_width = text_area.width as usize - 2 - '…'.len_utf8();
        let pattern = if self.edited_pattern.len() > max_text_width {
            format!(
                "…{}",
                &self.edited_pattern[self.edited_pattern.len() - max_text_width..]
            )
        } else {
            self.edited_pattern.clone()
        };

        let text = Text::from(Line::from(pattern.as_str()));
        let pattern_text = Paragraph::new(text);
        frame.render_widget(pattern_text, text_area);
        frame.set_cursor(
            std::cmp::min(
                text_area.x + pattern.len() as u16,
                text_area.x + text_area.width - 4,
            ),
            text_area.y,
        );
    }

    fn get_popup_area(frame_size: Rect, width_percent: u16) -> Rect {
        const POPUP_HEIGHT: u16 = 3;
        let top_bottom_margin = (frame_size.height - POPUP_HEIGHT) / 2;
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(top_bottom_margin),
                    Constraint::Length(POPUP_HEIGHT),
                    Constraint::Length(top_bottom_margin),
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
