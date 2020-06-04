use std::io;
use std::sync::mpsc;
use std::thread;

use termion::event::Key;
use termion::input::TermRead;

use grep::{
    matcher::LineTerminator,
    regex::RegexMatcher,
    searcher::{Searcher, SearcherBuilder, Sink, SinkMatch},
};
use ignore::WalkBuilder;

use crate::entries::{FileEntry, Match};

struct SearchConfig {
    pattern: String,
    path: String, // // path: &str -> AsRef<Path>
}
struct PathSearcher {
    config: SearchConfig,
    tx: mpsc::Sender<Event>,
}

impl PathSearcher {
    fn new(tx: mpsc::Sender<Event>, config: SearchConfig) -> Self {
        PathSearcher { config, tx }
    }

    fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let matcher = RegexMatcher::new_line_matcher(&self.config.pattern)?;
        let builder = WalkBuilder::new(&self.config.path);

        let walk_parallel = builder.build_parallel();
        walk_parallel.run(move || {
            let tx = self.tx.clone();
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
                        let line_number = sink_match.line_number().unwrap();
                        let text = std::str::from_utf8(sink_match.bytes()).unwrap_or("Not UTF-8");
                        let m = Match::new(line_number, text);
                        matches_in_entry.push(m);
                        Ok(true)
                    }),
                );

                if !matches_in_entry.is_empty() {
                    tx.send(Event::NewEntry(FileEntry {
                        name: String::from(dir_entry.path().to_str().unwrap()),
                        matches: matches_in_entry,
                    }))
                    .unwrap();
                }

                ignore::WalkState::Continue
            })
        });

        Ok(())
    }
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

pub enum Event {
    Input(Key),
    NewEntry(FileEntry),
    SearcherFinished,
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let _ = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
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

        let ps = PathSearcher::new(
            tx.clone(),
            SearchConfig {
                pattern: "kernel".into(),
                path: "/home/konrad/".into(),
            },
        );
        let _ = {
            thread::spawn(move || {
                ps.run();
                tx.send(Event::SearcherFinished)
            })
        };

        Events { rx }
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}
