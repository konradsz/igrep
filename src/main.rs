use std::io;

use clap::{App, Arg};

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text},
    Terminal,
};

mod entries;
mod event;
mod result_list;

use event::{Event, Events};
use result_list::ResultList;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("ig")
        .about("Interactive Grep")
        .arg(
            Arg::with_name("PATTERN")
                .help("Pattern to search")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("PATH")
                .help("Path to search")
                .required(true)
                .index(2),
        )
        .get_matches();

    let pattern = matches.value_of("PATTERN").unwrap();
    let path = matches.value_of("PATH").unwrap();

    //search_path(pattern, path)?;

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    // App
    let mut result_list = ResultList::new();
    result_list.add_entry(entries::FileEntry::new(
        "File A",
        vec![entries::Match::new(0, "m1"), entries::Match::new(0, "m2")],
    ));
    result_list.add_entry(entries::FileEntry::new(
        "File B",
        vec![entries::Match::new(0, "m3"), entries::Match::new(0, "m4")],
    ));

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            let header_style = Style::default().fg(Color::Red);

            let files_list = result_list
                .entries
                .iter()
                .map(|item| item.list())
                .flatten()
                .map(|e| match e {
                    entries::Type::Header(h) => Text::Styled(h.into(), header_style),
                    entries::Type::Match(n, t) => Text::raw(format!("{}: {}", n, t)),
                });
            let list_widget = List::new(files_list)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().modifier(Modifier::ITALIC))
                .highlight_symbol(">>");

            f.render_stateful_widget(list_widget, chunks[0], &mut result_list.state);
        })?;

        match events.next()? {
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
                _ => {}
            },
            Event::NewEntry => {
                result_list.add_entry(entries::FileEntry::new(
                    "New entry",
                    vec![entries::Match::new(0, "m1")],
                ));
            }
            Event::SearcherFinished => {
                result_list.add_entry(entries::FileEntry::new(
                    "FINISH",
                    vec![entries::Match::new(0, "finished")],
                ));
            }
        }
    }

    Ok(())
}
