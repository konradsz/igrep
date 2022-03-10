use anyhow::Result;
use args::Args;
use clap::Parser;
use std::io::Write;
use ui::editor::Editor;

mod args;
mod file_entry;
mod grep_match;
pub mod ig;
pub mod ui;

fn main() -> Result<()> {
    let args = Args::parse();

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

    let path = args.path.unwrap_or_else(|| "./".into());

    let search_config = ig::SearchConfig::from(args.pattern.unwrap(), path)?
        .case_insensitive(args.ignore_case)
        .case_smart(args.smart_case)
        .search_hidden(args.search_hidden)
        .globs(args.glob)?
        .file_types(args.type_matching, args.type_not)?;

    let mut app = ui::App::new(search_config, Editor::determine(args.editor.editor)?);
    app.run()?;

    Ok(())
}
