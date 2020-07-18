use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{error::Error, io::Write};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text},
    Frame, Terminal,
};

use super::input_handler::InputHandler;
use super::scroll_offset_list::{List, ListState, ScrollOffset};

use crate::ig::EntryType;
use crate::ig::Ig;
use crate::ig::SearchConfig;

pub struct App {
    ig: Ig,
    input_handler: InputHandler,
    result_list_state: ListState,
}

impl App {
    pub fn new(pattern: &str, path: &str) -> Self {
        Self {
            ig: Ig::new(SearchConfig {
                pattern: pattern.into(),
                path: path.into(),
            }),
            input_handler: InputHandler::default(),
            result_list_state: ListState::default(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.ig.search();

        loop {
            let backend = CrosstermBackend::new(std::io::stdout());
            let mut terminal = Terminal::new(backend)?;
            terminal.hide_cursor()?;

            enable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                EnterAlternateScreen,
                DisableMouseCapture
            )?;

            while self.ig.is_searching() || self.ig.is_idle() {
                terminal.draw(|mut f| self.draw(&mut f))?;

                self.ig.handle_searcher_event(); // this function could handle error event
                self.input_handler.handle_input(&mut self.ig)?;
            }

            self.ig.open_file_if_requested();

            if self.ig.exit_requested() {
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                disable_raw_mode()?;
                break;
            }
        }

        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(f.size());

        self.draw_list(f, chunks[0]);
        self.draw_footer(f, chunks[1]);
    }

    fn draw_list(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let width = f.size().width as usize;
        let header_style = Style::default().fg(Color::Red);

        let files_list = self.ig.result_list.iter().map(|e| match e {
            EntryType::Header(h) => Text::Styled(h.into(), header_style),
            EntryType::Match(n, t) => {
                let text = format!(" {}: {}", n, t);
                let text = format!("{: <1$}", text, width);
                Text::raw(text)
            }
        });

        let list_widget = List::new(files_list)
            .block(
                Block::default()
                    .title("List")
                    .borders(Borders::ALL)
                    .border_type(tui::widgets::BorderType::Rounded),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .scroll_offset(ScrollOffset { top: 1, bottom: 0 });

        self.result_list_state
            .select(self.ig.result_list.get_state().selected());
        f.render_stateful_widget(list_widget, area, &mut self.result_list_state);
    }

    fn draw_footer(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let current_match_index = self.ig.result_list.get_current_match_index();

        let app_status_color = if self.ig.is_searching() {
            Color::LightRed
        } else {
            Color::Green
        };
        let app_status = vec![Text::styled(
            if self.ig.is_searching() {
                "SEARCHING"
            } else {
                "FINISHED"
            },
            Style::default()
                .modifier(Modifier::BOLD)
                .bg(app_status_color)
                .fg(Color::Black),
        )];

        let search_result = if self.ig.is_searching() {
            Vec::default()
        } else {
            let total_no_of_matches = self.ig.result_list.get_total_number_of_matches();
            let message = if total_no_of_matches == 0 {
                " No matches found.".into()
            } else {
                let no_of_files = self.ig.result_list.get_total_number_of_file_entries();

                let matches_str = if total_no_of_matches == 1 {
                    "match"
                } else {
                    "matches"
                };
                let files_str = if no_of_files == 1 { "file" } else { "files" };

                let filtered_count = self.ig.result_list.get_filtered_matches_count();
                let filtered_str = if filtered_count != 0 {
                    format!(" ({} filtered out)", filtered_count)
                } else {
                    String::default()
                };

                format!(
                    " Found {} {} in {} {}{}.",
                    total_no_of_matches, matches_str, no_of_files, files_str, filtered_str
                )
            };

            vec![Text::styled(
                message,
                Style::default().bg(Color::DarkGray).fg(Color::Black),
            )]
        };

        let current_no_of_matches = self.ig.result_list.get_current_number_of_matches();
        let selected_info_text = format!("{}/{} ", current_match_index, current_no_of_matches);
        let selected_info_length = selected_info_text.len();

        let selected_info = vec![Text::styled(
            selected_info_text,
            Style::default().bg(Color::DarkGray).fg(Color::Black),
        )];

        let hsplit = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(12),
                    Constraint::Min(1),
                    Constraint::Length(selected_info_length as u16),
                ]
                .as_ref(),
            )
            .split(area);

        f.render_widget(
            Paragraph::new(app_status.iter())
                .style(Style::default().bg(app_status_color))
                .alignment(Alignment::Center),
            hsplit[0],
        );

        f.render_widget(
            Paragraph::new(search_result.iter())
                .style(Style::default().bg(Color::DarkGray))
                .alignment(Alignment::Left),
            hsplit[1],
        );

        f.render_widget(
            Paragraph::new(selected_info.iter())
                .style(Style::default().bg(Color::DarkGray))
                .alignment(Alignment::Right),
            hsplit[2],
        );
    }
}
