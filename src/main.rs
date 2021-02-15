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
                .help("Searches case insensitively."),
        )
        .arg(
            clap::Arg::with_name("smart-case")
                .long("smart-case")
                .short("S")
                .help("Searches case insensitively if the pattern is all lowercase. Search case sensitively otherwise."),
        )
        .arg(
            clap::Arg::with_name("TYPE")
                .long("type")
                .short("t")
                .help("Only search files matching TYPE. Multiple type flags may be provided.")
                .takes_value(true)
                .multiple(true)
        )
        .arg(
            clap::Arg::with_name("TYPE-NOT")
                .long("type-not")
                .short("T")
                .help("Do not search files matching TYPE. Multiple type-not flags may be provided.")
                .takes_value(true)
                .multiple(true)
        )
        .get_matches();

    let pattern = matches.value_of("PATTERN").unwrap();
    let path = if let Some(p) = matches.value_of("PATH") {
        p
    } else {
        "./"
    };

    let search_config = ig::SearchConfig::from(pattern, path)
        .case_insensitive(matches.is_present("ignore-case"))
        .case_smart(matches.is_present("smart-case"))
        .file_types(
            matches.values_of("TYPE").unwrap_or_default().collect(),
            matches.values_of("TYPE-NOT").unwrap_or_default().collect(),
        );

    let mut app = ui::App::new(search_config);
    app.run()?;

    Ok(())
}
