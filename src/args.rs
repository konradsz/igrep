use crate::ui::{editor::Editor, theme::ThemeVariant};
use clap::{Arg, ArgGroup, CommandFactory, Parser};
use std::{
    collections::HashSet,
    ffi::OsString,
    fs::File,
    io::{self, BufRead, BufReader},
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
        let (supported_long, supported_short) = Self::collect_supported_options();

        let path = "./config"; // Path
        match File::open(&path) {
            Ok(file) => parse_reader(file, supported_long, supported_short),
            Err(err) => panic!(),
        }
    }

    fn collect_supported_options() -> (HashSet<String>, HashSet<String>) {
        let app = Args::command();
        let to_exclude: Vec<_> = app.get_positionals().map(Arg::get_id).collect();

        let supported_long = app
            .get_arguments()
            .map(Arg::get_id)
            .filter(|arg| !to_exclude.contains(arg))
            .map(String::from)
            .collect();
        let supported_short = app
            .get_arguments()
            .filter_map(Arg::get_short)
            .map(String::from)
            .collect();

        (supported_long, supported_short)
    }
}

fn parse_reader<R: io::Read>(
    reader: R,
    supported_long: HashSet<String>,
    supported_short: HashSet<String>,
) -> impl Iterator<Item = OsString> {
    let reader = BufReader::new(reader);
    let mut args = vec![];
    let mut ignore_next_line = false;

    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with("--") {
            let el = line.split_terminator('=').next().unwrap();
            let el = el.strip_prefix("--").unwrap();
            if supported_long.contains(el) {
                args.push(OsString::from(line));
            } else {
                if !line.contains('=') {
                    ignore_next_line = true;
                }
            }
        } else if line.starts_with('-') {
            let el = line.split_terminator('=').next().unwrap();
            let el = el.strip_prefix('-').unwrap();
            if supported_short.contains(el) {
                args.push(OsString::from(line));
            } else {
                if !line.contains('=') {
                    ignore_next_line = true;
                }
            }
        } else {
            if ignore_next_line {
                ignore_next_line = false;
                continue;
            }
            args.push(OsString::from(line));
        }
    }

    // println!("{args:?}");
    args.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test2() {
        let supported_long = HashSet::from(["sup".to_owned()]);
        let supported_short = HashSet::from(["s".to_owned()]);

        let input = "\
# comment
--sup=value\n\r\
-s  \n\
value
--unsup

    # --comment
    value
        -s";
        let args = parse_reader(input.as_bytes(), supported_long, supported_short)
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(args, ["--sup=value", "-s", "value", "-s"]);
    }

    //     #[test]
    //     fn foo() {
    //         let supported_long = HashSet::from([
    //             "context".to_owned(),
    //             "smart-case".to_owned(),
    //             "bar".to_owned(),
    //             "foo".to_owned(),
    //         ]);
    //         let supported_short = HashSet::new();
    //         let errs = parse_reader(
    //             &b"\
    // # Test
    // --context=0
    //    --smart-case
    // -u

    //    # --bar
    // --foo
    // "[..],
    //             supported_long,
    //             supported_short,
    //         );
    //     }
}
