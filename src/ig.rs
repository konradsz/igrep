use crossterm::{
    event::{poll, read, DisableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{error::Error, io::Write, process::Command, sync::mpsc, time::Duration};

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text},
    Frame, Terminal,
};

use crate::entries::{EntryType, FileEntry};
use crate::result_list::ResultList;
use crate::scroll_offset_list::{
    List as ScrollOffsetList, ListState as ScrollOffsetListState, ScrollOffset,
};
use crate::searcher::{SearchConfig, Searcher};

#[derive(PartialEq)]
enum AppState {
    Idle,
    Searching,
    OpenFile(bool),
    Exit,
}

pub enum AppEvent {
    NewEntry(FileEntry),
    SearchingFinished,
}

pub struct Ig {
    rx: mpsc::Receiver<AppEvent>,
    searcher: Searcher,
    result_list: ResultList,
    result_list_state: ScrollOffsetListState,
    state: AppState,
    poll_timeout: u64,
}

impl Ig {
    pub fn new(pattern: &str, path: &str) -> Self {
        let (tx, rx) = mpsc::channel();

        let s = Searcher::new(
            SearchConfig {
                pattern: pattern.into(),
                path: path.into(),
            },
            tx.clone(),
        );

        Self {
            rx,
            searcher: s,
            result_list: ResultList::default(),
            result_list_state: ScrollOffsetListState::default(),
            state: AppState::Idle,
            poll_timeout: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.search();
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
            match self.state {
                AppState::Idle | AppState::Searching => continue,
                AppState::OpenFile(idle) => {
                    if let Some((file_name, line_number)) = self.result_list.get_selected_entry() {
                        let mut child_process = Command::new("nvim")
                            .arg(file_name)
                            .arg(format!("+{}", line_number))
                            .spawn()
                            .expect("Error: Failed to run editor");
                        child_process.wait()?;
                        self.state = if idle {
                            AppState::Idle
                        } else {
                            AppState::Searching
                        };
                    }
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

    fn search(&mut self) {
        self.state = AppState::Searching;
        self.searcher.search();
    }

    fn draw_and_handle_events(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        while self.state == AppState::Idle || self.state == AppState::Searching {
            terminal.draw(|mut f| self.draw(&mut f))?;

            self.handle_app_event(); // this function could handle error event
            self.handle_input()?;
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
        let header_style = Style::default().fg(Color::Red);

        let files_list = self.result_list.iter().map(|e| match e {
            EntryType::Header(h) => Text::Styled(h.into(), header_style),
            EntryType::Match(n, t) => Text::raw(format!("{}: {}", n, t)),
        });

        let list_widget = ScrollOffsetList::new(files_list)
            .block(
                Block::default()
                    .title("List")
                    .borders(Borders::ALL)
                    .border_type(tui::widgets::BorderType::Rounded),
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .scroll_offset(ScrollOffset { top: 1, bottom: 0 });

        self.result_list_state
            .select(self.result_list.get_state().selected());
        f.render_stateful_widget(list_widget, area, &mut self.result_list_state);
    }

    fn draw_footer(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let current_match_index = self.result_list.get_current_match_index();
        let no_of_matches = self.result_list.get_number_of_matches();

        let text_items = match self.state {
            AppState::Searching => vec![Text::styled(
                "Searching...",
                Style::default().bg(Color::DarkGray).fg(Color::White),
            )],
            _ => {
                let message = if no_of_matches == 0 {
                    " No matches found.".into()
                } else {
                    let no_of_files = self.result_list.get_number_of_file_entries();

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
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                )]
            }
        };

        let selected_info = format!("{}/{} ", current_match_index, no_of_matches);
        let selected_info_length = selected_info.len();

        let selected_info_items = vec![Text::styled(
            selected_info,
            Style::default().bg(Color::DarkGray).fg(Color::White),
        )];

        let hsplit = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length(selected_info_length as u16),
                ]
                .as_ref(),
            )
            .split(area);

        f.render_widget(
            Paragraph::new(text_items.iter())
                .style(Style::default().bg(Color::DarkGray))
                .alignment(Alignment::Left),
            hsplit[0],
        );

        f.render_widget(
            Paragraph::new(selected_info_items.iter())
                .style(Style::default().bg(Color::DarkGray))
                .alignment(Alignment::Right),
            hsplit[1],
        );
    }

    fn handle_app_event(&mut self) {
        if let Ok(event) = self.rx.try_recv() {
            match event {
                AppEvent::NewEntry(e) => self.result_list.add_entry(e),
                AppEvent::SearchingFinished => {
                    self.state = AppState::Idle;
                    self.poll_timeout = 1000;
                }
            }
        }
    }

    fn handle_input(&mut self) -> Result<(), Box<dyn Error>> {
        if poll(Duration::from_millis(self.poll_timeout))? {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                }) => self.result_list.next_match(),
                Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                }) => self.result_list.previous_match(),
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    ..
                }) => self.result_list.next_file(),
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('h'),
                    ..
                }) => self.result_list.previous_file(),
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    self.state = AppState::OpenFile(self.state == AppState::Idle);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => self.state = AppState::Exit,
                _ => (),
            }
        }

        Ok(())
    }
}
