mod entries;
mod ig;
mod input_handler;
mod result_list;
mod scroll_offset_list;
mod searcher;

use clap::{App, Arg};
use ig::App as IgApp;

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

    let mut ig = IgApp::new(pattern, path);
    ig.run()?;

    Ok(())
}
