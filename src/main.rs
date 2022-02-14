use anyhow::Result;
use std::io::Write;

mod file_entry;
mod grep_match;
mod ig;
mod ui;

fn main() -> Result<()> {
    let matches = clap::App::new("ig")
        .about("Interactive Grep")
        .author("Konrad Szymoniak <szymoniak.konrad@gmail.com>")
        .arg(
            clap::Arg::with_name("PATTERN")
                .help("Pattern to search")
                .required_unless("TYPE-LIST")
                .index(1),
        )
        .arg(
            clap::Arg::with_name("PATH")
                .help("Path to search")
                .required(false)
                .index(2),
        )
        .arg(
            clap::Arg::with_name("IGNORE-CASE")
                .long("ignore-case")
                .short("i")
                .help("Searches case insensitively."),
        )
        .arg(
            clap::Arg::with_name("SMART-CASE")
                .long("smart-case")
                .short("S")
                .help("Searches case insensitively if the pattern is all lowercase. Search case sensitively otherwise."),
        )
        .arg(
            clap::Arg::with_name("GLOB")
                .long("glob")
                .short("g")
                .help("Include files and directories for searching that match the given glob. Multiple globs may be provided.")
                .takes_value(true)
                .multiple(true)
        )
        .arg(
            clap::Arg::with_name("TYPE-LIST")
                .long("type-list")
                .help("Show all supported file types and their corresponding globs.")
        )
        .arg(
            clap::Arg::with_name("TYPE")
                .long("type")
                .short("t")
                .help("Only search files matching TYPE. Multiple types may be provided.")
                .takes_value(true)
                .multiple(true)
        )
        .arg(
            clap::Arg::with_name("TYPE-NOT")
                .long("type-not")
                .short("T")
                .help("Do not search files matching TYPE. Multiple types-not may be provided.")
                .takes_value(true)
                .multiple(true)
        )
        .get_matches();

    if matches.is_present("TYPE-LIST") {
        use itertools::Itertools;
        let mut builder = ignore::types::TypesBuilder::new();
        builder.add_defaults();
        for definition in builder.definitions() {
            writeln!(
                std::io::stdout(),
                "{}: {}",
                definition.name(),
                definition.globs().iter().format(", "),
            )?;
        }
        return Ok(());
    }

    let pattern = matches.value_of("PATTERN").expect("PATTERN is required");
    let path = if let Some(p) = matches.value_of("PATH") {
        p
    } else {
        "./"
    };

    let search_config = ig::SearchConfig::from(pattern, path)?
        .case_insensitive(matches.is_present("IGNORE-CASE"))
        .case_smart(matches.is_present("SMART-CASE"))
        .globs(matches.values_of("GLOB").unwrap_or_default().collect())?
        .file_types(
            matches.values_of("TYPE").unwrap_or_default().collect(),
            matches.values_of("TYPE-NOT").unwrap_or_default().collect(),
        )?;

    let mut app = ui::App::new(search_config, ui::editor::Editor::Neovim);
    app.run()?;

    Ok(())
}
