use anyhow::Result;
use args::Args;
use std::io::Write;
use ui::{
    editor::{self},
    theme::{dark::Dark, light::Light, Theme, ThemeVariant},
    App,
};

mod args;
mod file_entry;
mod grep_match;
pub mod ig;
pub mod ui;

fn main() -> Result<()> {
    let args = Args::parse_cli_and_config_file();

    if args.type_list {
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

    let paths = if args.paths.is_empty() {
        vec!["./".into()]
    } else {
        args.paths
    };

    let search_config = ig::SearchConfig::from(args.pattern.unwrap(), paths)?
        .case_insensitive(args.ignore_case)
        .case_smart(args.smart_case)
        .search_hidden(args.search_hidden)
        .globs(args.glob)?
        .file_types(args.type_matching, args.type_not)?;

    let theme: Box<dyn Theme> = match args.theme {
        ThemeVariant::Light => Box::new(Light),
        ThemeVariant::Dark => Box::new(Dark),
    };
    let mut app = App::new(
        search_config,
        editor::determine(args.editor.custom_command, args.editor.editor)?,
        theme,
    );
    app.run()?;

    Ok(())
}
