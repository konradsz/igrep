use anyhow::Result;
use clap::ArgEnum;
use clap::{ArgGroup, Parser};
use std::io::Write;
use ui::editor::Editor;

mod file_entry;
mod grep_match;
mod ig;
mod ui;

#[derive(Parser, Debug)]
struct EditorOpt {
    #[clap(env = "IGREP_EDITOR", long, arg_enum)]
    editor: Option<ui::editor::Editor>,
}

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
    path: Option<String>, // TODO: use PathBuf
    /// Text editor used to open selected match.
    #[clap(
        env = "IGREP_EDITOR",
        long,
        arg_enum,
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
    /// Search hidden files and directories.
    /// By default, hidden files and directories are skipped.
    #[clap(short = '.', long = "hidden")]
    search_hidden: bool,
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

    let path = args.path.unwrap_or_else(|| "./".into());

    let search_config = ig::SearchConfig::from(args.pattern.unwrap(), path)?
        .case_insensitive(args.ignore_case)
        .case_smart(args.smart_case)
        .search_hidden(args.search_hidden)
        .globs(args.glob)?
        .file_types(args.type_matching, args.type_not)?;

    let mut app = ui::App::new(search_config, args.editor);
    app.run()?;

    Ok(())
}

// TODO: improve error message
// TODO: define constants for env variables
fn determine_editor(editor_cli: Option<Editor>) -> Result<Editor> {
    if let Some(editor_cli) = editor_cli {
        Ok(editor_cli)
    } else if let Ok(igrep_editor_env) = std::env::var("IGREP_EDITOR") {
        // Editor::value_variants()
        Editor::from_str(&igrep_editor_env, false).map_err(|e| anyhow::anyhow!(e))
    } else if let Ok(editor_env) = std::env::var("EDITOR") {
        Editor::from_str(&editor_env, false).map_err(anyhow::Error::msg)
    } else {
        Ok(Editor::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use test_case::test_case;

    lazy_static! {
        static ref SERIAL_TEST: std::sync::Mutex<()> = Default::default();
    }

    #[test_case(Some("nano"), Some("vim"), Some("neovim") => matches Ok(Editor::Nano))]
    #[test_case(None, Some("nano"), Some("neovim") => matches Ok(Editor::Nano))]
    #[test_case(None, None, Some("nano") => matches Ok(Editor::Nano))]
    #[test_case(Some("unsupported-editor"), None, None => matches Err(_))]
    #[test_case(None, Some("unsupported-editor"), None => matches Err(_))]
    #[test_case(None, None, Some("unsupported-editor") => matches Err(_))]
    #[test_case(None, None, None => matches Ok(Editor::Vim))]
    fn test_editor_options_precedence(
        cli_option: Option<&str>,
        igrep_editor_env: Option<&str>,
        editor_env: Option<&str>,
    ) -> Result<Editor> {
        let _guard = SERIAL_TEST.lock().unwrap();
        std::env::remove_var("IGREP_EDITOR");
        std::env::remove_var("EDITOR");

        let opt = if let Some(cli_option) = cli_option {
            EditorOpt::try_parse_from(&["test", "--editor", cli_option])
        } else {
            EditorOpt::try_parse_from(&["test"])
        };

        if let Some(igrep_editor_env) = igrep_editor_env {
            std::env::set_var("IGREP_EDITOR", igrep_editor_env);
        }

        if let Some(editor_env) = editor_env {
            std::env::set_var("EDITOR", editor_env);
        }

        determine_editor(opt?.editor)
    }
}
