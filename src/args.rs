use crate::ui::editor::Editor;
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

pub const IGREP_EDITOR_ENV: &str = "IGREP_EDITOR";
pub const EDITOR_ENV: &str = "EDITOR";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
            ArgGroup::new("excl")
                .args(&["pattern", "type-list"])
                .required(true)
))]
pub struct Args {
    /// Regular expression used for searching.
    pub pattern: Option<String>,
    /// File or directory to search. Directories are searched recursively.
    /// If not specified, searching starts from current directory.
    pub path: Option<PathBuf>,
    #[clap(flatten)]
    pub editor: EditorOpt,
    /// Searches case insensitively.
    #[clap(short = 'i', long)]
    pub ignore_case: bool,
    /// Searches case insensitively if the pattern is all lowercase.
    /// Search case sensitively otherwise.
    #[clap(short = 'S', long)]
    pub smart_case: bool,
    /// Search hidden files and directories.
    /// By default, hidden files and directories are skipped.
    #[clap(short = '.', long = "hidden")]
    pub search_hidden: bool,
    /// Include files and directories for searching that match the given glob.
    /// Multiple globs may be provided.
    #[clap(short, long)]
    pub glob: Vec<String>,
    /// Show all supported file types and their corresponding globs.
    #[clap(long)]
    pub type_list: bool,
    /// Only search files matching TYPE. Multiple types may be provided.
    #[clap(short = 't', long = "type")]
    pub type_matching: Vec<String>,
    /// Do not search files matching TYPE-NOT. Multiple types-not may be provided.
    #[clap(short = 'T', long)]
    pub type_not: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct EditorOpt {
    /// Text editor used to open selected match.
    #[clap(long, arg_enum)]
    pub editor: Option<Editor>,
}
