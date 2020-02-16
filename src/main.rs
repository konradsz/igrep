extern crate clap;
extern crate grep;
extern crate ignore;

use std::sync::mpsc;

use clap::{App, Arg};
use ignore::WalkBuilder;

use grep::matcher::LineTerminator;
use grep::regex::RegexMatcher;
use grep::searcher::{Searcher, SearcherBuilder, Sink, SinkMatch};

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

fn search_path(pattern: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    let matcher = RegexMatcher::new_line_matcher(&pattern)?;
    let builder = WalkBuilder::new(path);

    builder.build_parallel().run(move || {
        let tx = tx.clone();
        let matcher = matcher.clone();
        let mut searcher = SearcherBuilder::new()
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

            let _ = searcher.search_path(
                &matcher,
                dir_entry.path(),
                MatchesSink(|_, sink_match| {
                    let m = Match {
                        line_number: sink_match.line_number().unwrap(),
                        text: String::from(std::str::from_utf8(sink_match.bytes()).unwrap()),
                    };
                    matches_in_entry.push(m);
                    Ok(true)
                }),
            );

            if !matches_in_entry.is_empty() {
                tx.send(FileMatch {
                    name: String::from(dir_entry.path().to_str().unwrap()),
                    matches: matches_in_entry,
                }).unwrap();
            }

            ignore::WalkState::Continue
        })
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

    search_path(pattern, path)?;

    Ok(())
}
