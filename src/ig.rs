use crossterm::{
    event::{self, poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::process::Command;
use std::{
    error::Error,
    io::{stdout, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListState, Text},
    Terminal,
};

use crate::searcher::Searcher;

pub struct Ig {
    rx: mpsc::Receiver<String>,
    result_list: Vec<String>,
}

impl Ig {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let mut s = Searcher::new(tx);
        let _ = {
            thread::spawn(move || {
                s.run(); // handle error?
            })
        };

        Self {
            rx,
            result_list: Vec::default(),
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
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        enable_raw_mode()?;
        execute!(terminal.backend_mut(), EnterAlternateScreen)?;

        //let mut result_list = Vec::new();

        loop {
            self.draw_list(&mut terminal)?;
            match self.rx.recv() {
                // try_recv?
                Ok(s) => self.result_list.push(s),
                Err(e) => (),
            };
            if poll(Duration::from_millis(0))? {
                // change timeout after finding everything
                let event = read()?;
                if event == Event::Key(KeyCode::Char('e').into()) {
                    return Ok(Some(String::from("file_name")));
                } else if event == Event::Key(KeyCode::Char('q').into()) {
                    break;
                }
            }
        }

        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(None)
    }

    fn draw_list(
        &self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn Error>> {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            //let header_style = Style::default().fg(Color::Red);

            //let files = vec![Text::raw("item1"), Text::raw("item2")];
            let files = self.result_list.iter().map(|s| Text::raw(s));
            //let files_list = files.into_iter();

            let list_widget = List::new(files)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_stateful_widget(list_widget, chunks[0], &mut ListState::default());
        })?;

        Ok(())
    }
}
