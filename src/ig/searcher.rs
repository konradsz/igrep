use super::{file_entry::FileEntry, sink::MatchesSink, SearchConfig};
use grep::{
    matcher::LineTerminator,
    regex::RegexMatcherBuilder,
    searcher::{BinaryDetection, SearcherBuilder},
};
use ignore::WalkBuilder;
use std::{path::Path, sync::mpsc};

pub enum Event {
    NewEntry(FileEntry),
    SearchingFinished,
    Error,
}

pub fn search(config: SearchConfig, tx: mpsc::Sender<Event>) {
    std::thread::spawn(move || {
        let path_searchers = config
            .paths
            .clone()
            .into_iter()
            .map(|path| {
                let config = config.clone();
                let tx = tx.clone();
                std::thread::spawn(move || run(&path, config, tx))
            })
            .collect::<Vec<_>>();

        for searcher in path_searchers {
            if searcher.join().is_err() {
                tx.send(Event::Error).ok();
                return;
            }
        }

        tx.send(Event::SearchingFinished).ok();
    });
}

fn run(path: &Path, config: SearchConfig, tx: mpsc::Sender<Event>) {
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
        .word(config.word_regexp)
        .build(&config.pattern)
        .expect("Cannot build RegexMatcher");

    let mut builder = WalkBuilder::new(path);
    let walker = builder
        .overrides(config.overrides.clone())
        .types(config.types.clone())
        .hidden(!config.search_hidden)
        .follow_links(config.follow_links);

    let sorted = true;
    if !sorted
    {
        let walk_parallel = walker
            .build_parallel();

        walk_parallel.run(move || {
            let tx = tx.clone();
            let matcher = matcher.clone();
            let mut grep_searcher = grep_searcher.clone();

            Box::new(move |result| {
                let dir_entry = match result {
                    Ok(entry) => {
                        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
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
    }
    else
    {
        let walk_sorted = walker
            .sort_by_file_name(|a,b| a.cmp(b));

        for result in walk_sorted.build(){
            let tx = tx.clone();
            let matcher = matcher.clone();
            let mut grep_searcher = grep_searcher.clone();

                let dir_entry = match result {
                    Ok(entry) => {
                        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                            continue;
                        }
                        entry
                    }
                    Err(_) => continue,
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

                continue
        }
    }
}
