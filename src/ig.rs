use crossterm::{
    event::{poll, read, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{error::Error, io::Write, process::Command, sync::mpsc, thread, time::Duration};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text},
    Terminal,
};

use crate::entries::{EntryType, FileEntry};
use crate::result_list::ResultList;
use crate::searcher::{SearchConfig, Searcher};

pub struct Ig {
    rx: mpsc::Receiver<FileEntry>,
    result_list: ResultList,
}

impl Ig {
    pub fn new(pattern: &str, path: &str) -> Self {
        let (tx, rx) = mpsc::channel();

        let mut s = Searcher::new(
            SearchConfig {
                pattern: pattern.into(),
                path: path.into(),
            },
            tx,
        );
        let _ = {
            thread::spawn(move || {
                s.run(); // handle error?
            })
        };

        Self {
            rx,
            result_list: ResultList::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.draw_and_handle_events()? {
                Some(_file_name) => {
                    let mut child_process = Command::new("nvim")
                        .spawn()
                        .expect("Error: Failed to run editor");
                    child_process.wait()?;
                }
                None => break,
            }
        }

        Ok(())
    }

    fn draw_and_handle_events(&mut self) -> Result<Option<String>, Box<dyn Error>> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        enable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            DisableMouseCapture
        )?;

        loop {
            self.draw_list(&mut terminal)?;

            match self.rx.try_recv() {
                Ok(s) => self.result_list.add_entry(s),
                Err(_e) => (),
            };

            if poll(Duration::from_millis(0))? {
                match read()? {
                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        ..
                    }) => self.result_list.next(),
                    Event::Key(KeyEvent {
                        code: KeyCode::Up, ..
                    }) => self.result_list.previous(),
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    }) => return Ok(Some(String::from("file_name"))),
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        ..
                    }) => break,
                    _ => (),
                }
            }
        }

        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        disable_raw_mode()?;

        Ok(None)
    }

    fn draw_list(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let header_style = Style::default().fg(Color::Red);

            let files_list = self
                .result_list
                .entries
                .iter()
                .map(|item| item.list())
                .flatten()
                .map(|e| match e {
                    EntryType::Header(h) => Text::Styled(h.into(), header_style),
                    EntryType::Match(n, t) => Text::raw(format!("{}: {}", n, t)),
                });

            let list_widget = List::new(files_list)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_stateful_widget(list_widget, chunks[0], &mut self.result_list.state);
        })?;

        Ok(())
    }
}
