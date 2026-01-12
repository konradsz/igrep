use crate::{
    editor::Editor,
    ui::{context_viewer::ContextViewerPosition, theme::ThemeVariant},
};
use clap::{CommandFactory, Parser, ValueEnum};
use std::{
    ffi::OsString,
    fs::File,
    io::{self, BufRead, BufReader},
    iter::once,
    path::PathBuf,
};

pub const IGREP_CUSTOM_EDITOR_ENV: &str = "IGREP_CUSTOM_EDITOR";
pub const IGREP_EDITOR_ENV: &str = "IGREP_EDITOR";
pub const EDITOR_ENV: &str = "EDITOR";
pub const RIPGREP_CONFIG_PATH_ENV: &str = "RIPGREP_CONFIG_PATH";
pub const VISUAL_ENV: &str = "VISUAL";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Regular expression used for searching.
    #[arg(group = "pattern_or_type", required = true)]
    pub pattern: Option<String>,
    /// Files or directories to search. Directories are searched recursively.
    /// If not specified, searching starts from current directory.
    pub paths: Vec<PathBuf>,
    #[clap(flatten)]
    pub editor: EditorOpt,
    /// UI color theme.
    #[clap(long, default_value_t)]
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
    /// Follow symbolic links while traversing directories.
    #[clap(short = 'L', long = "follow")]
    pub follow_links: bool,
    /// Only show matches surrounded by word boundaries.
    #[clap(short = 'w', long = "word-regexp")]
    pub word_regexp: bool,
    /// Exact matches with no regex. Useful when searching for a string full of delimiters.
    #[clap(short = 'F', long = "fixed-strings")]
    pub fixed_strings: bool,
    /// Search with pattern contains newline character ('\n').
    #[clap(short = 'U', long = "multiline")]
    pub multi_line: bool,
    /// Include files and directories for searching that match the given glob.
    /// Multiple globs may be provided.
    #[clap(short, long)]
    pub glob: Vec<String>,
    /// Show all supported file types and their corresponding globs.
    #[arg(group = "pattern_or_type", required = true)]
    #[clap(long)]
    pub type_list: bool,
    /// Only search files matching TYPE. Multiple types may be provided.
    #[clap(short = 't', long = "type")]
    pub type_matching: Vec<String>,
    /// Do not search files matching TYPE-NOT. Multiple types-not may be provided.
    #[clap(short = 'T', long)]
    pub type_not: Vec<String>,
    /// Context viewer position at startup
    #[clap(long, value_enum, default_value_t = ContextViewerPosition::None)]
    pub context_viewer: ContextViewerPosition,
    /// Sort results, see ripgrep for details
    #[clap(long = "sort")]
    pub sort_by: Option<SortKeyArg>,
    /// Sort results reverse, see ripgrep for details
    #[clap(long = "sortr")]
    pub sort_by_reverse: Option<SortKeyArg>,
}

#[derive(Parser, Debug)]
pub struct EditorOpt {
    /// Text editor used to open selected match.
    #[arg(group = "editor_command")]
    #[clap(long)]
    pub editor: Option<Editor>,

    /// Custom command used to open selected match. Must contain {file_name} and {line_number} tokens.
    #[arg(group = "editor_command")]
    #[clap(long, env = IGREP_CUSTOM_EDITOR_ENV)]
    pub custom_command: Option<String>,
}

#[derive(Clone, ValueEnum, Debug, PartialEq)]
pub enum SortKeyArg {
    Path,
    Modified,
    Created,
    Accessed,
}

impl Args {
    pub fn parse_cli_and_config_file() -> Self {
        // first validate if CLI arguments are valid
        Args::parse_from(std::env::args_os());

        let mut args_os: Vec<_> = std::env::args_os().collect();
        let to_ignore = args_os
            .iter()
            .filter_map(|arg| {
                let arg = arg.to_str().expect("Not valid UTF-8");
                arg.starts_with('-')
                    .then(|| arg.trim_start_matches('-').to_owned())
            })
            .collect::<Vec<_>>();

        // then extend them with those from config file
        args_os.extend(Self::parse_config_file(to_ignore));

        Args::parse_from(args_os)
    }

    fn parse_config_file(to_ignore: Vec<String>) -> Vec<OsString> {
        match std::env::var_os(RIPGREP_CONFIG_PATH_ENV) {
            None => Vec::default(),
            Some(config_path) => match File::open(config_path) {
                Ok(file) => {
                    let supported_arguments = Self::collect_supported_arguments();
                    let to_ignore = Self::pair_ignored(to_ignore, &supported_arguments);
                    Self::parse_from_reader(file, supported_arguments, to_ignore)
                }
                Err(_) => Vec::default(),
            },
        }
    }

    fn pair_ignored(
        to_ignore: Vec<String>,
        supported_arguments: &[(Option<String>, Option<String>)],
    ) -> Vec<String> {
        to_ignore
            .iter()
            .filter(|i| {
                supported_arguments
                    .iter()
                    .any(|arg| arg.0.as_ref() == Some(i) || arg.1.as_ref() == Some(i))
            })
            .flat_map(|i| {
                match supported_arguments
                    .iter()
                    .find(|arg| arg.0.as_ref() == Some(i) || arg.1.as_ref() == Some(i))
                {
                    Some(arg) => Box::new(once(arg.0.clone()).chain(once(arg.1.clone())))
                        as Box<dyn Iterator<Item = _>>,
                    None => Box::new(once(None)),
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
                    ignore_next_line = false;
                    let long = long.split_terminator('=').next().expect("Empty line");
                    if supported.iter().any(|el| el.0 == Some(long.to_string()))
                        && !to_ignore.contains(&long.to_owned())
                    {
                        return Some(OsString::from(line));
                    }
                    if !line.contains('=') {
                        ignore_next_line = true;
                    }
                    None
                } else if let Some(short) = line.strip_prefix('-') {
                    ignore_next_line = false;
                    let short = short.split_terminator('=').next().expect("Empty line");
                    if supported.iter().any(|el| el.1 == Some(short.to_string()))
                        && !to_ignore.contains(&short.to_owned())
                    {
                        return Some(OsString::from(line));
                    }
                    if !line.contains('=') {
                        ignore_next_line = true;
                    }
                    None
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
    fn ripgrep_example_config() {
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
    fn trim_whitespaces() {
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
    fn skip_line_after_ignored_option() {
        let supported_args = vec![
            (Some("aaa".to_owned()), Some("a".to_owned())),
            (Some("bbb".to_owned()), Some("b".to_owned())),
        ];

        let input = "\
        --aaa
        value
        --bbb
        value
        ";
        let args = Args::parse_from_reader(
            input.as_bytes(),
            supported_args.clone(),
            vec!["aaa".to_owned()],
        )
        .into_iter()
        .map(|s| s.into_string().unwrap())
        .collect::<Vec<_>>();
        assert_eq!(args, ["--bbb", "value"]);

        let input = "\
        -a
        value
        -b
        value
        ";
        let args = Args::parse_from_reader(input.as_bytes(), supported_args, vec!["a".to_owned()])
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(args, ["-b", "value"]);
    }

    #[test]
    fn do_not_skip_line_after_ignored_option_if_value_inline() {
        let supported_args = vec![
            (Some("aaa".to_owned()), Some("a".to_owned())),
            (Some("bbb".to_owned()), Some("b".to_owned())),
        ];

        let input = "\
        --aaa=value
        --bbb
        value
        ";
        let args = Args::parse_from_reader(
            input.as_bytes(),
            supported_args.clone(),
            vec!["aaa".to_owned()],
        )
        .into_iter()
        .map(|s| s.into_string().unwrap())
        .collect::<Vec<_>>();
        assert_eq!(args, ["--bbb", "value"]);

        let input = "\
        -a=value
        -b
        value
        ";
        let args = Args::parse_from_reader(input.as_bytes(), supported_args, vec!["a".to_owned()])
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(args, ["-b", "value"]);
    }

    #[test]
    fn do_not_skip_line_after_ignored_flag() {
        let supported_args = vec![
            (Some("aaa".to_owned()), Some("a".to_owned())),
            (Some("bbb".to_owned()), Some("b".to_owned())),
        ];

        let input = "\
        --aaa
        --bbb
        value
        ";
        let args = Args::parse_from_reader(
            input.as_bytes(),
            supported_args.clone(),
            vec!["aaa".to_owned()],
        )
        .into_iter()
        .map(|s| s.into_string().unwrap())
        .collect::<Vec<_>>();
        assert_eq!(args, ["--bbb", "value"]);

        let input = "\
        -a
        -b
        value
        ";
        let args = Args::parse_from_reader(input.as_bytes(), supported_args, vec!["a".to_owned()])
            .into_iter()
            .map(|s| s.into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(args, ["-b", "value"]);
    }

    #[test]
    fn pair_ignored() {
        let to_ignore = Args::pair_ignored(
            vec![
                "a".to_owned(),
                "bbb".to_owned(),
                "ddd".to_owned(),
                "e".to_owned(),
            ],
            &[
                (Some("aaa".to_owned()), Some("a".to_owned())),
                (Some("bbb".to_owned()), Some("b".to_owned())),
                (Some("ccc".to_owned()), Some("c".to_owned())),
                (Some("ddd".to_owned()), None),
                (None, Some("e".to_owned())),
            ],
        );

        let extended: HashSet<String> = HashSet::from_iter(to_ignore);
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
