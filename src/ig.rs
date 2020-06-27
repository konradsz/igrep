use crossterm::{
    event::{poll, read, DisableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{error::Error, io::Write, process::Command, sync::mpsc, thread, time::Duration};

use tui::{backend::CrosstermBackend, Terminal};

use crate::entries::FileEntry;
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
                // handle error?
                match s.run() {
                    Ok(_) => (),
                    Err(_) => (),
                }
            })
        };

        Self {
            rx,
            result_list: ResultList::default(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.draw_and_handle_events()? {
                Some((file_name, line_number)) => {
                    let mut child_process = Command::new("nvim")
                        .arg(file_name)
                        .arg(format!("+{}", line_number))
                        .spawn()
                        .expect("Error: Failed to run editor");
                    child_process.wait()?;
                }
                None => break,
            }
        }

        Ok(())
    }

    fn draw_and_handle_events(&mut self) -> Result<Option<(&str, u64)>, Box<dyn Error>> {
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
            self.result_list.render(&mut terminal)?;

            match self.rx.try_recv() {
                Ok(entry) => self.result_list.add_entry(entry),
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
                    }) => {
                        if self.result_list.is_empty() {
                            continue;
                        } else {
                            return Ok(self.result_list.get_selected_entry());
                        }
                    }
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
}
