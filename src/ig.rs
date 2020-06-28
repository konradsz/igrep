use crossterm::{
    event::{poll, read, DisableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{error::Error, io::Write, process::Command, sync::mpsc, thread, time::Duration};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Paragraph, Text},
    Frame, Terminal,
};

use crate::entries::{EntryType, FileEntry};
use crate::result_list::ResultList;
use crate::searcher::{SearchConfig, Searcher};

pub enum IgEvent {
    NewEntry(FileEntry),
    SearchingFinished,
}

pub struct Ig {
    rx: mpsc::Receiver<IgEvent>,
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
                tx.send(IgEvent::SearchingFinished);
            })
        };

        Self {
            rx,
            result_list: ResultList::default(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
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

            match self.draw_and_handle_events(&mut terminal)? {
                Some((file_name, line_number)) => {
                    let mut child_process = Command::new("nvim")
                        .arg(file_name)
                        .arg(format!("+{}", line_number))
                        .spawn()
                        .expect("Error: Failed to run editor");
                    child_process.wait()?;
                }
                None => {
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
    ) -> Result<Option<(&str, u64)>, Box<dyn Error>> {
        loop {
            terminal.draw(|mut f| self.draw(&mut f))?;

            match self.rx.try_recv() {
                Ok(event) => match event {
                    IgEvent::NewEntry(e) => self.result_list.add_entry(e),
                    IgEvent::SearchingFinished => (),
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
                        if !self.result_list.is_empty() {
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

        Ok(None)
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

        let files_list = self.result_list.entries.iter().map(|e| match e {
            EntryType::Header(h) => Text::Styled(h.into(), header_style),
            EntryType::Match(n, t) => Text::raw(format!("{}: {}", n, t)),
        });

        let list_widget = List::new(files_list)
            .block(Block::default().title("List").borders(Borders::NONE))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().modifier(Modifier::ITALIC))
            .highlight_symbol(">>");

        f.render_stateful_widget(list_widget, area, &mut self.result_list.state);
    }

    fn draw_footer(&mut self, f: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: Rect) {
        let text_items = vec![Text::styled(
            "Footer",
            Style::default().bg(Color::DarkGray).fg(Color::White),
        )];
        f.render_widget(
            Paragraph::new(text_items.iter()).style(Style::default().bg(Color::DarkGray)),
            area,
        );
    }
}
