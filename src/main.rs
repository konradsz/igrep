extern crate clap;
extern crate ignore;

use clap::{App, Arg};
use ignore::WalkBuilder;

fn search_path(path: &str) {
    let builder = WalkBuilder::new(path);

    builder.build_parallel().run(|| {
        Box::new(move |result| {
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
            return ignore::WalkState::Continue;
        })
    });
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

    search_path(path);
}
