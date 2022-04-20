use crate::ui::{editor::Editor, theme::ThemeVariant};
use clap::{ArgGroup, CommandFactory, Parser};
use std::{
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
    pub fn parse_cli_and_config_file() -> Self {
        // first validate if CLI arguments are valid
        Args::parse_from(std::env::args_os());

        // then extend them with those from config file
        let mut args_os: Vec<_> = std::env::args_os().collect();
        let to_ignore = args_os
            .iter()
            .filter_map(|arg| {
                let arg = arg.to_str().expect("Not valid UTF-8");
                // arg.st
                if arg.starts_with("--") || arg.starts_with('-') {
                    Some(arg.trim_start_matches('-').to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        args_os.extend(Self::parse_config_file(to_ignore));

        Args::parse_from(args_os)
    }

    fn parse_config_file(to_ignore: Vec<String>) -> Vec<OsString> {
        let supported_arguments = Self::collect_supported_arguments();
        let to_ignore = Self::extend_ignored(to_ignore, &supported_arguments);

        let path = "./config"; // Path
        match File::open(&path) {
            Ok(file) => Self::parse_from_reader(file, supported_arguments, to_ignore),
            Err(_) => Vec::default(),
        }
    }

    fn extend_ignored(
        to_ignore: Vec<String>,
        supported_arguments: &Vec<(Option<String>, Option<String>)>,
    ) -> Vec<String> {
        to_ignore
            .iter()
            .flat_map(|i| {
                match supported_arguments.iter().find(|arg| {
                    if let Some(l) = &arg.0 {
                        if l == i {
                            return true;
                        }
                    }
                    if let Some(s) = &arg.1 {
                        if s == i {
                            return true;
                        }
                    }
                    false
                }) {
                    Some(asd) => Box::new(
                        std::iter::once(asd.0.clone()).chain(std::iter::once(asd.1.clone())),
                    ) as Box<dyn Iterator<Item = _>>,
                    None => Box::new(std::iter::once(None)),
                }
            })
            .flatten()
            .collect()
    }

    fn collect_supported_arguments() -> Vec<(Option<String>, Option<String>)> {
        Args::command()
            .get_arguments()
            .filter_map(|arg| match (arg.get_long(), arg.get_short()) {
                (None, None) => None,
                (l, s) => Some((l.map(|l| l.to_string()), s.map(|s| s.to_string()))),
            })
            .collect::<Vec<_>>()
    }

    fn parse_from_reader<R: io::Read>(
        reader: R,
        supported: Vec<(Option<String>, Option<String>)>,
        to_ignore: Vec<String>,
    ) -> Vec<OsString> {
        let reader = BufReader::new(reader);
        let mut ignore_next_line = false;

        reader
            .lines()
            .filter_map(|line| {
                let line = line.expect("Not valid UTF-8");
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }

                if let Some(long) = line.strip_prefix("--") {
                    let long = long.split_terminator('=').next().expect("Empty line");
                    if supported.iter().any(|el| el.0 == Some(long.to_owned())) {
                        if to_ignore.contains(&long.to_owned()) {
                            None
                        } else {
                            Some(OsString::from(line))
                        }
                    } else {
                        if !line.contains('=') {
                            ignore_next_line = true;
                        }
                        None
                    }
                } else if let Some(short) = line.strip_prefix('-') {
                    let short = short.split_terminator('=').next().expect("Empty line");
                    if supported.iter().any(|el| el.1 == Some(short.to_string())) {
                        if to_ignore.contains(&short.to_owned()) {
                            None
                        } else {
                            Some(OsString::from(line))
                        }
                    } else {
                        if !line.contains('=') {
                            ignore_next_line = true;
                        }
                        None
                    }
                } else {
                    if ignore_next_line {
                        ignore_next_line = false;
                        return None;
                    }
                    Some(OsString::from(line))
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test1() {
        let supported_args = vec![
            (Some("glob".to_owned()), Some("g".to_owned())),
            (Some("smart-case".to_owned()), None),
        ];
        let input = "\
            # Don't let ripgrep vomit really long lines to my terminal, and show a preview.
            --max-columns=150
            --max-columns-preview

            # Add my 'web' type.
            --type-add
            web:*.{html,css,js}*

            # Using glob patterns to include/exclude files or folders
            -g=!git/*

            # or
            --glob
            !git/*

            # Set the colors.
            --colors=line:none
            --colors=line:style:bold

            # Because who cares about case!?
            --smart-case";

        let args = Args::parse_from_reader(input.as_bytes(), supported_args, vec![])
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(args, ["-g=!git/*", "--glob", "!git/*", "--smart-case"]);
    }

    #[test]
    fn test2() {
        let supported_args = vec![(Some("sup".to_owned()), Some("s".to_owned()))];

        let input = "\
    # comment
    --sup=value\n\r\
    -s  \n\
    value
    --unsup

        # --comment
        value
            -s";
        let args = Args::parse_from_reader(input.as_bytes(), supported_args, vec![])
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(args, ["--sup=value", "-s", "value", "-s"]);
    }

    #[test]
    fn ignore_explicit() {
        let supported_args = vec![(Some("sup".to_owned()), Some("s".to_owned()))];

        let input = "--sup";
        let args =
            Args::parse_from_reader(input.as_bytes(), supported_args, vec!["sup".to_owned()])
                .into_iter()
                .map(|s| s.into_string().unwrap())
                .collect::<Vec<_>>();
        assert!(args.is_empty());
    }

    #[test]
    fn ignore_implicit() {
        let to_ignore = Args::extend_ignored(
            vec![
                "a".to_owned(),
                "bbb".to_owned(),
                "ddd".to_owned(),
                "e".to_owned(),
            ],
            &vec![
                (Some("aaa".to_owned()), Some("a".to_owned())),
                (Some("bbb".to_owned()), Some("b".to_owned())),
                (Some("ccc".to_owned()), Some("c".to_owned())),
                (Some("ddd".to_owned()), None),
                (None, Some("e".to_owned())),
            ],
        );

        let extended: HashSet<String> = HashSet::from_iter(to_ignore.into_iter());
        let expected: HashSet<String> = HashSet::from([
            "aaa".to_owned(),
            "a".to_owned(),
            "bbb".to_owned(),
            "b".to_owned(),
            "ddd".to_owned(),
            "e".to_owned(),
        ]);

        assert_eq!(extended, expected);
    }
}
