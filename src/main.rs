use std::sync::mpsc;

use clap::{App, Arg};
use ignore::WalkBuilder;

use std::thread;

use grep::matcher::LineTerminator;
use grep::regex::RegexMatcher;
use grep::searcher::{Searcher, SearcherBuilder, Sink, SinkMatch};

mod entries;
mod event;
mod result_list;

use event::{Event, Events};
use result_list::ResultList;

#[derive(Debug)]
struct Match {
    line_number: u64,
    text: String,
}

#[derive(Debug)]
struct FileMatch {
    name: String,
    matches: Vec<Match>,
}

// path: &str -> AsRef<Path>
fn search_path(pattern: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    let matcher = RegexMatcher::new_line_matcher(&pattern)?;
    let builder = WalkBuilder::new(path);

    let walk_parallel = builder.build_parallel();
    let handle = thread::spawn(|| {
        walk_parallel.run(move || {
            let tx = tx.clone();
            let matcher = matcher.clone();
            let mut searcher = SearcherBuilder::new()
                //.binary_detection(BinaryDetection::quit(b'\x00')) // from simplegrep - check it
                .line_terminator(LineTerminator::byte(b'\n'))
                .line_number(true)
                .multi_line(false)
                .build();

            Box::new(move |result| {
                let dir_entry = match result {
                    Err(err) => {
                        eprintln!("{}", err);
                        return ignore::WalkState::Continue;
                    }
                    Ok(entry) => {
                        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                            return ignore::WalkState::Continue;
                        }
                        entry
                    }
                };

                let mut matches_in_entry = Vec::new();

                // handle error like in simplegrep
                let _ = searcher.search_path(
                    &matcher,
                    dir_entry.path(),
                    MatchesSink(|_, sink_match| {
                        //println!("{}", dir_entry.path().to_str().unwrap());
                        let m = Match {
                            line_number: sink_match.line_number().unwrap(),
                            text: std::str::from_utf8(sink_match.bytes())
                                .map_or(String::from("Not UTF-8"), |s| String::from(s)),
                        };
                        matches_in_entry.push(m);
                        Ok(true)
                    }),
                );

                if !matches_in_entry.is_empty() {
                    tx.send(FileMatch {
                        name: String::from(dir_entry.path().to_str().unwrap()),
                        matches: matches_in_entry,
                    })
                    .unwrap();
                }

                ignore::WalkState::Continue
            })
        });
    });

    for received in rx {
        println!("{:?}", received);
    }

    Ok(())
}

pub struct MatchesSink<F>(pub F)
where
    F: FnMut(&Searcher, &SinkMatch) -> Result<bool, std::io::Error>;

impl<F> Sink for MatchesSink<F>
where
    F: FnMut(&Searcher, &SinkMatch) -> Result<bool, std::io::Error>,
{
    type Error = std::io::Error;

    fn matched(
        &mut self,
        searcher: &Searcher,
        sink_match: &SinkMatch,
    ) -> Result<bool, std::io::Error> {
        (self.0)(searcher, sink_match)
    }
}

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, Text},
    Terminal,
};

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
        vec![entries::Match::new("m1"), entries::Match::new("m2")],
    ));
    result_list.add_entry(entries::FileEntry::new(
        "File B",
        vec![entries::Match::new("m3"), entries::Match::new("m4")],
    ));

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            //let style = Style::default().fg(Color::White).bg(Color::Black);
            let files_list = result_list
                .entries
                .iter()
                .map(|item| item.list())
                .flatten()
                .map(|e| Text::raw(e));
            let list = List::new(files_list)
                .block(Block::default().title("List").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
            /*let items = List::new(items)
            .block(Block::default().borders(Borders::NONE).title("List"))
            .style(style)
            .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD));*/
            //.highlight_symbol(">");
            f.render_stateful_widget(list, chunks[0], &mut result_list.state);
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
            Event::Tick => {
                //app.advance();
            }
        }
    }

    Ok(())
}
