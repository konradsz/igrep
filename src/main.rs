extern crate clap;
extern crate ignore;
extern crate grep;

use clap::{App, Arg};
use ignore::WalkBuilder;

use grep::regex::RegexMatcher;
use grep::matcher::{LineTerminator, Matcher};
use grep::searcher::{SearcherBuilder, Searcher, SinkMatch, Sink};

fn search_path(pattern: &str, path: &str) -> Result<(), Box<std::error::Error>> {
    let builder = WalkBuilder::new(path);

    let matcher = RegexMatcher::new_line_matcher(&pattern)?;

    builder.build_parallel().run(|| {
        Box::new(move |result| {
            let matcher = matcher.clone();
            let mut searcher = SearcherBuilder::new()
                .line_terminator(LineTerminator::byte(b'\n'))
                .line_number(true)
                .multi_line(false)
                .build();

            let dir_entry = match result {
                Err(err) => {
                    eprintln!("{}", err);
                    return ignore::WalkState::Continue;
                }
                Ok(entry) => {
                    if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                        println!("!file: {}", entry.path().display());
                        return ignore::WalkState::Continue;
                    } else {
                        println!("file: {}", entry.path().display());
                    }
                    entry
                }
            };
            let mut match_count = 0;
            let result = searcher.search_path(
                &matcher,
                dir_entry.path(),
                MatchesSink(|_, sink_match| {
                    matcher.find_iter(sink_match.bytes(), |_| {
                        match_count += 1;
                        true
                    })?;
                    Ok(true)
                }),
            );

            ignore::WalkState::Continue
        })
    });

    Ok(())
}

pub struct MatchesSink<F>(pub F) where F: FnMut(&Searcher, &SinkMatch) -> Result<bool, std::io::Error>;

impl<F> Sink for MatchesSink<F> where F: FnMut(&Searcher, &SinkMatch) -> Result<bool, std::io::Error> {
    type Error = std::io::Error;
    fn matched(&mut self, searcher: &Searcher, sink_match: &SinkMatch) -> Result<bool, std::io::Error> {
        (self.0)(searcher, sink_match)
    }
}

fn main() {
    let matches = App::new("ig")
        .about("Interactive Grep")
        .arg(
            Arg::with_name("PATH")
                .help("Path to search")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = matches.value_of("PATH").unwrap();

    search_path("dupa", path);
}
