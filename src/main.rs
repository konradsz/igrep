use anyhow::Result;
use clap::{ArgGroup, Parser};
use std::io::Write;

mod file_entry;
mod grep_match;
mod ig;
mod ui;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
            ArgGroup::new("excl")
                .args(&["pattern", "type-list"])
                .required(true)
))]
struct Args {
    /// Regular expression used for searching.
    pattern: Option<String>,
    /// File or directory to search. Directories are searched recursively.
    /// If not specified, searching starts from current directory.
    path: Option<String>,
    /// Text editor used to open selected match.
    #[clap(
        long,
        arg_enum,
        env = "IGREP_EDITOR",
        default_value_t = ui::editor::Editor::Vim
    )]
    editor: ui::editor::Editor,
    /// Searches case insensitively.
    #[clap(short = 'i', long)]
    ignore_case: bool,
    /// Searches case insensitively if the pattern is all lowercase.
    /// Search case sensitively otherwise.
    #[clap(short = 'S', long)]
    smart_case: bool,
    /// Include files and directories for searching that match the given glob.
    /// Multiple globs may be provided.
    #[clap(short, long)]
    glob: Vec<String>,
    /// Show all supported file types and their corresponding globs.
    #[clap(long)]
    type_list: bool,
    /// Only search files matching TYPE. Multiple types may be provided.
    #[clap(short = 't', long = "type")]
    type_matching: Vec<String>,
    /// Do not search files matching TYPE-NOT. Multiple types-not may be provided.
    #[clap(short = 'T', long)]
    type_not: Vec<String>,
}

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

    let path = args.path.unwrap_or("./".into());

    let search_config = ig::SearchConfig::from(args.pattern.unwrap(), path)?
        .case_insensitive(args.ignore_case)
        .case_smart(args.smart_case)
        .globs(args.glob)?
        .file_types(args.type_matching, args.type_not)?;

    let mut app = ui::App::new(search_config, args.editor);
    app.run()?;

    Ok(())
}
