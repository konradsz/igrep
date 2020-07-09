mod app;
mod entries;
mod ig;
mod input_handler;
mod result_list;
mod scroll_offset_list;
mod searcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::App::new("ig")
        .about("Interactive Grep")
        .arg(
            clap::Arg::with_name("PATTERN")
                .help("Pattern to search")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("PATH")
                .help("Path to search")
                .required(true)
                .index(2),
        )
        .get_matches();

    let pattern = matches.value_of("PATTERN").unwrap();
    let path = matches.value_of("PATH").unwrap();

    let mut app = app::App::new(pattern, path);
    app.run()?;

    Ok(())
}
