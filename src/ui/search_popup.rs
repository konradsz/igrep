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
}

impl SearchPopup {
    pub fn new(pattern: String) -> Self {
        Self {
            visible: false,
            pattern,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn draw(&self, frame: &mut Frame<CrosstermBackend<std::io::Stdout>>) {
        if !self.visible {
            return;
        }

        let block = Block::default()
            .title("Pattern")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        let area = Self::get_popup_area(frame.size(), 50);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let text = Text::from(Line::from(self.pattern.as_str()));
        let pattern_text = Paragraph::new(text);
        let mut text_area = area.clone();
        text_area.y += 1; // one line below the border
        text_area.x += 2; // two chars to the right
        frame.render_widget(pattern_text, text_area);
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
