mod ig;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap::App::new("ig")
        .about("Interactive Grep")
        .author("Konrad Szymoniak <szymoniak.konrad@gmail.com>")
        .arg(
            clap::Arg::with_name("PATTERN")
                .help("Pattern to search")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("PATH")
                .help("Path to search")
                .required(false)
                .index(2),
        )
        .arg(
            clap::Arg::with_name("ignore-case")
                .long("ignore-case")
                .short("i")
                .help("Perform case insensitive search"),
        )
        .get_matches();

    let pattern = matches.value_of("PATTERN").unwrap();
    let path = if let Some(p) = matches.value_of("PATH") {
        p
    } else {
        "./"
    };

    let search_config =
        ig::SearchConfig::from(pattern, path).case_insensitive(matches.is_present("ignore-case"));

    let mut app = ui::app::App::new(search_config);
    app.run()?;

    Ok(())
}
