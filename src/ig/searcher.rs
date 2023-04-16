use anyhow::Result;
use std::sync::mpsc;

use super::{sink::MatchesSink, SearchConfig};
use crate::file_entry::FileEntry;
use grep::{
    matcher::LineTerminator,
    regex::RegexMatcherBuilder,
    searcher::{BinaryDetection, SearcherBuilder},
};
use ignore::WalkBuilder;

pub enum Event {
    NewEntry(FileEntry),
    SearchingFinished,
    Error,
}

pub struct Searcher {
    config: SearchConfig,
    tx: mpsc::Sender<Event>,
}

impl Searcher {
    pub fn new(config: SearchConfig, tx: mpsc::Sender<Event>) -> Self {
        Self { config, tx }
    }

    pub fn search(&self) {
        let tx = self.tx.clone();
        let config = self.config.clone();
        let _ = std::thread::spawn(move || {
            if Self::run(config.clone(), tx.clone()).is_err() {
                tx.send(Event::Error).ok();
            }

            tx.send(Event::SearchingFinished).ok();
        });
    }

    fn run(config: SearchConfig, tx: mpsc::Sender<Event>) -> Result<()> {
        let grep_searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_terminator(LineTerminator::byte(b'\n'))
            .line_number(true)
            .multi_line(false)
            .build();

        let matcher = RegexMatcherBuilder::new()
            .line_terminator(Some(b'\n'))
            .case_insensitive(config.case_insensitive)
            .case_smart(config.case_smart)
            .build(&config.pattern)?;
        let mut builder = WalkBuilder::new(&config.path);

        let walk_parallel = builder
            .overrides(config.overrides.clone())
            .types(config.types.clone())
            .hidden(!config.search_hidden)
            .build_parallel();
        walk_parallel.run(move || {
            let tx = tx.clone();
            let matcher = matcher.clone();
            let mut grep_searcher = grep_searcher.clone();

            Box::new(move |result| {
                let dir_entry = match result {
                    Ok(entry) => {
                        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                            return ignore::WalkState::Continue;
                        }
                        entry
                    }
                    Err(_) => return ignore::WalkState::Continue,
                };
                let mut matches_in_entry = Vec::new();
                let sr = MatchesSink::new(&matcher, &mut matches_in_entry);
                grep_searcher
                    .search_path(&matcher, dir_entry.path(), sr)
                    .ok();

                if !matches_in_entry.is_empty() {
                    tx.send(Event::NewEntry(FileEntry::new(
                        dir_entry.path().to_string_lossy().into_owned(),
                        matches_in_entry,
                    )))
                    .ok();
                }

                ignore::WalkState::Continue
            })
        });

        Ok(())
    }
}
