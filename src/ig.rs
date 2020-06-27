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

pub enum MyEvent {
    NewEntry(FileEntry),
    SearchingFinished,
}

pub struct Ig {
    rx: mpsc::Receiver<MyEvent>,
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
            tx.clone(),
        );
        let _ = {
            thread::spawn(move || {
                // handle error?
                match s.run() {
                    Ok(_) => (),
                    Err(_) => (),
                }
                tx.send(MyEvent::SearchingFinished);
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
                Ok(event) => match event {
                    MyEvent::NewEntry(e) => self.result_list.add_entry(e),
                    MyEvent::SearchingFinished => (),
                },
                Err(_) => (),
            };

            if poll(Duration::from_millis(0))? {
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
                        if self.result_list.is_empty() {
                            continue;
                        } else {
                            return Ok(self.result_list.get_selected_entry());
                        }
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc, ..
                    })
                    | Event::Key(KeyEvent {
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
