use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{error::Error, io::Write, process::Command};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text},
    Frame, Terminal,
};

use crate::entries::EntryType;
use crate::ig::Ig;
use crate::input_handler::InputHandler;
use crate::scroll_offset_list::{List, ScrollOffset};
use crate::searcher::SearchConfig;

#[derive(PartialEq)]
pub enum AppState {
    Idle,
    Searching,
    OpenFile(bool),
    Exit,
}

pub struct App {
    ig: Ig,
    input_handler: InputHandler,
}

impl App {
    pub fn new(pattern: &str, path: &str) -> Self {
        Self {
            ig: Ig::new(SearchConfig {
                pattern: pattern.into(),
                path: path.into(),
            }),
            input_handler: InputHandler::default(),
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

            self.draw_and_handle_events(&mut terminal)?;
            match self.ig.state {
                AppState::Idle | AppState::Searching => continue,
                AppState::OpenFile(idle) => {
                    if let Some((file_name, line_number)) = self.ig.result_list.get_selected_entry()
                    {
                        let mut child_process = Command::new("nvim")
                            .arg(file_name)
                            .arg(format!("+{}", line_number))
                            .spawn()
                            .expect("Error: Failed to run editor");
                        child_process.wait()?;
                    }

                    self.ig.state = if idle {
                        AppState::Idle
                    } else {
                        AppState::Searching
                    };
                }
                AppState::Exit => {
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    disable_raw_mode()?;
                    break;
                }
            }
        }

        Ok(())
    }

    fn draw_and_handle_events(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        while self.ig.state == AppState::Idle || self.ig.state == AppState::Searching {
            terminal.draw(|mut f| self.draw(&mut f))?;

            self.ig.handle_searcher_event(); // this function could handle error event
            self.input_handler.handle_input(&mut self.ig)?;
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

        self.ig
            .result_list_state
            .select(self.ig.result_list.get_state().selected());
        f.render_stateful_widget(list_widget, area, &mut self.ig.result_list_state);
    }

    fn draw_footer(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let current_match_index = self.ig.result_list.get_current_match_index();
        let no_of_matches = self.ig.result_list.get_number_of_matches();

        let app_status_color = match self.ig.state {
            AppState::Searching => Color::LightRed,
            _ => Color::Green,
        };
        let app_status = vec![Text::styled(
            match self.ig.state {
                AppState::Searching => "SEARCHING",
                _ => "FINISHED",
            },
            Style::default()
                .modifier(Modifier::BOLD)
                .bg(app_status_color)
                .fg(Color::Black),
        )];

        let search_result = match self.ig.state {
            AppState::Searching => vec![],
            _ => {
                let message = if no_of_matches == 0 {
                    " No matches found.".into()
                } else {
                    let no_of_files = self.ig.result_list.get_number_of_file_entries();

                    let matches_str = if no_of_matches == 1 {
                        "match"
                    } else {
                        "matches"
                    };
                    let files_str = if no_of_files == 1 { "file" } else { "files" };

                    format!(
                        " Found {} {} in {} {}.",
                        no_of_matches, matches_str, no_of_files, files_str
                    )
                };

                vec![Text::styled(
                    message,
                    Style::default().bg(Color::DarkGray).fg(Color::Black),
                )]
            }
        };

        let selected_info_text = format!("{}/{} ", current_match_index, no_of_matches);
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
