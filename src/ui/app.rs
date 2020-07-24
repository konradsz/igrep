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
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use super::input_handler::InputHandler;
use super::scroll_offset_list::{List, ListItem, ListState, ScrollOffset};

use crate::ig::EntryType;
use crate::ig::Ig;
use crate::ig::SearchConfig;

pub struct App {
    ig: Ig,
    input_handler: InputHandler,
    result_list_state: ListState,
}

impl App {
    pub fn new(config: SearchConfig) -> Self {
        Self {
            ig: Ig::new(config),
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

                self.ig.handle_searcher_event();
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
        let files_list: Vec<ListItem> = self
            .ig
            .result_list
            .iter()
            .map(|e| match e {
                EntryType::Header(h) => {
                    ListItem::new(Span::styled(h, Style::default().fg(Color::LightMagenta)))
                }
                EntryType::Match(n, t, offsets) => {
                    let line_number =
                        Span::styled(format!(" {}: ", n), Style::default().fg(Color::Green));

                    let mut spans = vec![line_number];

                    let mut current_position = 0;
                    for offset in offsets {
                        let before_match = Span::raw(&t[current_position..offset.0]);
                        let actual_match =
                            Span::styled(&t[offset.0..offset.1], Style::default().fg(Color::Red));

                        // sut current position to the end of current match
                        current_position = offset.1;

                        spans.push(before_match);
                        spans.push(actual_match);
                    }

                    // push remaining text of a line
                    spans.push(Span::raw(&t[current_position..]));

                    ListItem::new(Spans::from(spans))
                }
            })
            .collect();

        let list_widget = List::new(files_list)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(tui::widgets::BorderType::Rounded),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Rgb(58, 58, 58)))
            .scroll_offset(ScrollOffset::default().top(1).bottom(0));

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
        let app_status = Span::styled(
            if self.ig.is_searching() {
                "SEARCHING"
            } else {
                "FINISHED"
            },
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(app_status_color)
                .fg(Color::Black),
        );

        let search_result = if self.ig.is_searching() {
            Span::raw("")
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

            Span::styled(
                message,
                Style::default()
                    .bg(Color::Rgb(58, 58, 58))
                    .fg(Color::Rgb(147, 147, 147)),
            )
        };

        let current_no_of_matches = self.ig.result_list.get_current_number_of_matches();
        let selected_info_text = format!("{}/{} ", current_match_index, current_no_of_matches);
        let selected_info_length = selected_info_text.len();

        let selected_info = Span::styled(
            selected_info_text,
            Style::default()
                .bg(Color::Rgb(58, 58, 58))
                .fg(Color::Rgb(147, 147, 147)),
        );

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
            Paragraph::new(app_status)
                .style(Style::default().bg(app_status_color))
                .alignment(Alignment::Center),
            hsplit[0],
        );

        f.render_widget(
            Paragraph::new(search_result)
                .style(Style::default().bg(Color::Rgb(58, 58, 58)))
                .alignment(Alignment::Left),
            hsplit[1],
        );

        f.render_widget(
            Paragraph::new(selected_info)
                .style(Style::default().bg(Color::Rgb(58, 58, 58)))
                .alignment(Alignment::Right),
            hsplit[2],
        );
    }
}
