use anyhow::Result;
use std::sync::{mpsc, Arc};

use grep::{
    matcher::LineTerminator,
    matcher::Matcher,
    regex::RegexMatcherBuilder,
    searcher::{
        BinaryDetection, Searcher as GrepSearcher, SearcherBuilder as GrepSearcherBuilder, Sink,
        SinkMatch,
    },
};
use ignore::WalkBuilder;

use super::entries::{FileEntry, Match};
use super::SearchConfig;

pub enum Event {
    NewEntry(FileEntry),
    SearchingFinished,
    Error,
}

pub struct Searcher {
    inner: Arc<SearcherImpl>,
    tx: mpsc::Sender<Event>,
}

impl Searcher {
    pub fn new(config: SearchConfig, tx: mpsc::Sender<Event>) -> Self {
        Self {
            inner: Arc::new(SearcherImpl::new(config)),
            tx,
        }
    }

    pub fn search(&self) {
        let local_self = self.inner.clone();
        let tx_local = self.tx.clone();
        let _ = std::thread::spawn(move || {
            if local_self.run(tx_local.clone()).is_err() {
                tx_local.send(Event::Error).ok();
            }

            tx_local.send(Event::SearchingFinished).ok();
        });
    }
}

struct SearcherImpl {
    config: SearchConfig,
}

impl SearcherImpl {
    pub fn new(config: SearchConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, tx2: mpsc::Sender<Event>) -> Result<()> {
        let grep_searcher = GrepSearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_terminator(LineTerminator::byte(b'\n'))
            .line_number(true)
            .multi_line(false)
            .build();

        let matcher = RegexMatcherBuilder::new()
            .line_terminator(Some(b'\n'))
            .case_insensitive(self.config.case_insensitive)
            .case_smart(self.config.case_smart)
            .build(&self.config.pattern)?;
        let mut builder = WalkBuilder::new(&self.config.path);

        let walk_parallel = builder.types(self.config.types.clone()).build_parallel();
        walk_parallel.run(move || {
            let tx = tx2.clone();
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
                let sr = SinkRecorder::new(&matcher, &mut matches_in_entry);
                grep_searcher
                    .search_path(&matcher, dir_entry.path(), sr)
                    .ok();

                if !matches_in_entry.is_empty() {
                    tx.send(Event::NewEntry(FileEntry::new(
                        dir_entry.path().to_str().unwrap(),
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

struct SinkRecorder<'a, M>
where
    M: Matcher,
{
    matcher: M,
    matches_in_entry: &'a mut Vec<Match>,
}

impl<'a, M> SinkRecorder<'a, M>
where
    M: Matcher,
{
    fn new(matcher: M, matches_in_entry: &'a mut Vec<Match>) -> Self {
        Self {
            matcher,
            matches_in_entry,
        }
    }
}

impl<'a, M> Sink for SinkRecorder<'a, M>
where
    M: Matcher,
{
    type Error = std::io::Error;

    fn matched(
        &mut self,
        _: &GrepSearcher,
        sink_match: &SinkMatch,
    ) -> Result<bool, std::io::Error> {
        let line_number = sink_match.line_number().unwrap();
        let text = std::str::from_utf8(sink_match.bytes());

        let mut offsets = vec![];
        self.matcher
            .find_iter(sink_match.bytes(), |m| {
                offsets.push((m.start(), m.end()));
                true
            })
            .ok();

        if let Ok(t) = text {
            self.matches_in_entry
                .push(Match::new(line_number, t, offsets));
        };

        Ok(true)
    }
}
