use crate::ui::{editor::Editor, theme::ThemeVariant};
use clap::{Arg, ArgGroup, CommandFactory, Parser};
use std::{
    collections::HashSet,
    env::ArgsOs,
    ffi::{OsStr, OsString},
    fmt::Arguments,
    path::PathBuf,
};

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
    /// UI color theme.
    #[clap(long, arg_enum, default_value_t = ThemeVariant::Dark)]
    pub theme: ThemeVariant,
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

impl Args {
    pub fn parse_cli_args_and_config_file() -> Self {
        // validate if CLI arguments are ok
        Args::parse_from(std::env::args_os());

        let mut args_os: Vec<_> = std::env::args_os().collect();
        args_os.extend(Self::read_config_file());

        Args::parse_from(args_os)
    }

    pub fn read_config_file() -> impl Iterator<Item = OsString> {
        let app = Args::command();
        let to_exclude: Vec<_> = app.get_positionals().map(Arg::get_id).collect();
        let supported_long: HashSet<_> = app
            .get_arguments()
            .map(Arg::get_id)
            .filter(|arg| !to_exclude.contains(arg))
            .collect();

        let supported_short: HashSet<_> = app.get_arguments().filter_map(Arg::get_short).collect();

        for i in supported_long {
            println!("{i}");
        }

        for i in supported_short {
            println!("{i}");
        }

        vec!["a".into()].into_iter()
    }
}
