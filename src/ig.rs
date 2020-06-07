use std::process::Command;
use std::sync::mpsc;
use std::thread;

use termion::input::TermRead;

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text},
    Terminal,
};

use crate::entries::{EntryType, FileEntry, Match};
use crate::result_list::ResultList;
use crate::searcher::{SearchConfig, Searcher};

pub enum Event {
    Input(Key),
    NewEntry(FileEntry),
    SearcherFinished,
}

pub struct Ig {
    rx: mpsc::Receiver<Event>,
}

impl Ig {
    pub fn new(pattern: &str, path: &str) -> Self {
        let (tx, rx) = mpsc::channel();
        let _ = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = std::io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        if let Err(err) = tx.send(Event::Input(key)) {
                            eprintln!("{}", err);
                            return;
                        }
                    }
                }
            })
        };

        let ps = Searcher::new(
            tx.clone(),
            SearchConfig {
                pattern: pattern.into(),
                path: path.into(),
            },
        );
        let _ = {
            thread::spawn(move || {
                ps.run();
                tx.send(Event::SearcherFinished)
            })
        };

        Self { rx }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdout = std::io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        let mut result_list = ResultList::new();

        loop {
            terminal.draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .split(f.size());

                let header_style = Style::default().fg(Color::Red);

                let files_list = result_list
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

                f.render_stateful_widget(list_widget, chunks[0], &mut result_list.state);
            })?;

            match self.next()? {
                Event::Input(input) => match input {
                    Key::Char('q') => {
                        break;
                    }
                    Key::Down => {
                        result_list.next();
                    }
                    Key::Up => {
                        result_list.previous();
                    }
                    Key::Char('\n') => {
                        let mut child_process = Command::new("/usr/bin/sh")
                            .arg("-c")
                            .arg("nvim")
                            .spawn()
                            .expect("Error: Failed to run editor");
                        let stdin = std::io::stdin();
                        let _lock_handle = stdin.lock();
                        child_process.wait()?;

                        // workaround: force redraw of the terminal
                        let size = terminal.size().unwrap();
                        terminal.resize(size)?;
                        terminal.hide_cursor()?;
                    }
                    _ => {}
                },
                Event::NewEntry(entry) => {
                    result_list.add_entry(entry);
                }
                Event::SearcherFinished => {
                    result_list
                        .add_entry(FileEntry::new("FINISH", vec![Match::new(0, "finished")]));
                }
            }
        }
        Ok(())
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}
