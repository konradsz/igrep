use crate::args::{EDITOR_ENV, IGREP_EDITOR_ENV, VISUAL_ENV};
use anyhow::{anyhow, Result};
use clap::ValueEnum;
use itertools::Itertools;
use std::{
    fmt::{self, Debug, Display, Formatter},
    process::{Child, Command},
};
use strum::Display;

#[derive(Display, Default, PartialEq, Eq, Copy, Clone, Debug, ValueEnum)]
#[strum(serialize_all = "lowercase")]
pub enum Editor {
    #[default]
    Vim,
    Neovim,
    Nvim,
    Nano,
    Code,
    Vscode,
    CodeInsiders,
    Emacs,
    Emacsclient,
    Hx,
    Helix,
    Subl,
    SublimeText,
    Micro,
    Intellij,
    Goland,
    Pycharm,
    Less,
}

#[derive(Debug)]
pub enum EditorCommand {
    Builtin(Editor),
    Custom(String, String),
}

impl EditorCommand {
    pub fn new(custom_command: Option<String>, editor_cli: Option<Editor>) -> Result<Self> {
        if let Some(custom_command) = custom_command {
            let (program, args) = custom_command.split_once(' ').ok_or(
                anyhow!("Expected program and its arguments")
                    .context(format!("Incorrect editor command: '{custom_command}'")),
            )?;

            if args.matches("{file_name}").count() != 1 {
                return Err(anyhow!("Expected one occurrence of '{{file_name}}'.")
                    .context(format!("Incorrect editor command: '{custom_command}'")));
            }

            if args.matches("{line_number}").count() != 1 {
                return Err(anyhow!("Expected one occurrence of '{{line_number}}'.")
                    .context(format!("Incorrect editor command: '{custom_command}'")));
            }

            return Ok(EditorCommand::Custom(program.into(), args.into()));
        }

        let add_error_context = |e: String, env_value: String, env_name: &str| {
            let possible_variants = Editor::value_variants()
                .iter()
                .map(Editor::to_string)
                .join(", ");
            anyhow!(e).context(format!(
                "\"{env_value}\" read from ${env_name}, possible variants: [{possible_variants}]",
            ))
        };

        let read_from_env = |name| {
            std::env::var(name).ok().map(|value| {
                Editor::from_str(&extract_editor_name(&value), false)
                    .map_err(|error| add_error_context(error, value, name))
            })
        };

        Ok(EditorCommand::Builtin(
            editor_cli
                .map(Ok)
                .or_else(|| read_from_env(IGREP_EDITOR_ENV))
                .or_else(|| read_from_env(VISUAL_ENV))
                .or_else(|| read_from_env(EDITOR_ENV))
                .unwrap_or(Ok(Editor::default()))?,
        ))
    }

    pub fn spawn(&self, file_name: &str, line_number: u64) -> Result<Child> {
        let path = which::which(self.program())?;
        let mut command = Command::new(path);
        command.args(self.args(file_name, line_number));
        command.spawn().map_err(anyhow::Error::from)
    }

    fn program(&self) -> &str {
        match self {
            EditorCommand::Builtin(editor) => match editor {
                Editor::Vim => "vim",
                Editor::Neovim | Editor::Nvim => "nvim",
                Editor::Nano => "nano",
                Editor::Code | Editor::Vscode => "code",
                Editor::CodeInsiders => "code-insiders",
                Editor::Emacs => "emacs",
                Editor::Emacsclient => "emacsclient",
                Editor::Hx => "hx",
                Editor::Helix => "helix",
                Editor::Subl | Editor::SublimeText => "subl",
                Editor::Micro => "micro",
                Editor::Intellij => "idea",
                Editor::Goland => "goland",
                Editor::Pycharm => "pycharm",
                Editor::Less => "less",
            },
            EditorCommand::Custom(program, _) => program,
        }
    }

    fn args(&self, file_name: &str, line_number: u64) -> Box<dyn Iterator<Item = String>> {
        match self {
            EditorCommand::Builtin(editor) => match editor {
                Editor::Vim
                | Editor::Neovim
                | Editor::Nvim
                | Editor::Nano
                | Editor::Micro
                | Editor::Less => {
                    Box::new([format!("+{line_number}"), file_name.into()].into_iter())
                }
                Editor::Code | Editor::Vscode | Editor::CodeInsiders => {
                    Box::new(["-g".into(), format!("{file_name}:{line_number}")].into_iter())
                }
                Editor::Emacs | Editor::Emacsclient => Box::new(
                    ["-nw".into(), format!("+{line_number}"), file_name.into()].into_iter(),
                ),
                Editor::Hx | Editor::Helix | Editor::Subl | Editor::SublimeText => {
                    Box::new([format!("{file_name}:{line_number}")].into_iter())
                }
                Editor::Intellij | Editor::Goland | Editor::Pycharm => Box::new(
                    ["--line".into(), format!("{line_number}"), file_name.into()].into_iter(),
                ),
            },
            EditorCommand::Custom(_, args) => {
                let args = args.replace("{file_name}", file_name);
                let args = args.replace("{line_number}", &line_number.to_string());

                let args = args.split_whitespace().map(ToOwned::to_owned).collect_vec();
                Box::new(args.into_iter())
            }
        }
    }
}

impl Display for EditorCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.program())
    }
}

fn extract_editor_name(input: &str) -> String {
    let mut split = input.rsplit('/');
    split.next().unwrap().into()
}

#[cfg(test)]
mod tests {
    use super::EditorCommand::Builtin;
    use super::*;
    use crate::args::EditorOpt;
    use clap::Parser;
    use lazy_static::lazy_static;
    use test_case::test_case;

    lazy_static! {
        static ref SERIAL_TEST: std::sync::Mutex<()> = Default::default();
    }

    #[test_case("non_builtin_editor" => matches Err(_); "editor name only")]
    #[test_case("non_builtin_editor {file_name}" => matches Err(_); "no line number")]
    #[test_case("non_builtin_editor {line_number}" => matches Err(_); "no file name")]
    #[test_case("non_builtin_editor {file_name} {file_name} {line_number}" => matches Err(_); "file name twice")]
    #[test_case("non_builtin_editor {file_name} {line_number} {line_number}" => matches Err(_); "line number twice")]
    #[test_case("non_builtin_editor{file_name} {line_number}" => matches Err(_); "program not separated from arg")]
    #[test_case("non_builtin_editor {file_name}:{line_number}" => matches Ok(_); "correct command with one arg")]
    #[test_case("non_builtin_editor {file_name} {line_number}" => matches Ok(_); "correct command with two args")]
    fn parsing_custom_command(command: &str) -> Result<EditorCommand> {
        EditorCommand::new(Some(command.into()), None)
    }

    #[test_case(Some("nano"), Some("vim"), None, Some("neovim") => matches Ok(Builtin(Editor::Nano)); "cli")]
    #[test_case(None, Some("nano"), None, Some("neovim") => matches Ok(Builtin(Editor::Nano)); "igrep env")]
    #[test_case(None, None, Some("nano"), Some("helix") => matches Ok(Builtin(Editor::Nano)); "visual env")]
    #[test_case(None, None, None, Some("nano") => matches Ok(Builtin(Editor::Nano)); "editor env")]
    #[test_case(Some("unsupported-editor"), None, None, None => matches Err(_); "unsupported cli")]
    #[test_case(None, Some("unsupported-editor"), None, None => matches Err(_); "unsupported igrep env")]
    #[test_case(None, None, None, Some("unsupported-editor") => matches Err(_); "unsupported editor env")]
    #[test_case(None, None, None, None => matches Ok(Builtin(Editor::Vim)); "default editor")]
    #[test_case(None, Some("/usr/bin/nano"), None, None => matches Ok(Builtin(Editor::Nano)); "igrep env path")]
    #[test_case(None, None, None, Some("/usr/bin/nano") => matches Ok(Builtin(Editor::Nano)); "editor env path")]
    fn editor_options_precedence(
        cli_option: Option<&str>,
        igrep_editor_env: Option<&str>,
        visual_env: Option<&str>,
        editor_env: Option<&str>,
    ) -> Result<EditorCommand> {
        let _guard = SERIAL_TEST.lock().unwrap();
        std::env::remove_var(IGREP_EDITOR_ENV);
        std::env::remove_var(VISUAL_ENV);
        std::env::remove_var(EDITOR_ENV);

        let opt = if let Some(cli_option) = cli_option {
            EditorOpt::try_parse_from(["test", "--editor", cli_option])
        } else {
            EditorOpt::try_parse_from(["test"])
        };

        if let Some(igrep_editor_env) = igrep_editor_env {
            std::env::set_var(IGREP_EDITOR_ENV, igrep_editor_env);
        }

        if let Some(visual_env) = visual_env {
            std::env::set_var(VISUAL_ENV, visual_env);
        }

        if let Some(editor_env) = editor_env {
            std::env::set_var(EDITOR_ENV, editor_env);
        }

        EditorCommand::new(None, opt?.editor)
    }

    const FILE_NAME: &str = "file_name";
    const LINE_NUMBER: u64 = 123;

    #[test]
    fn custom_command() {
        let editor_command = EditorCommand::new(
            Some("non_builtin_editor -@{file_name} {line_number}".into()),
            None,
        )
        .unwrap();

        assert_eq!(editor_command.program(), "non_builtin_editor");
        assert_eq!(
            editor_command.args(FILE_NAME, LINE_NUMBER).collect_vec(),
            vec![format!("-@{FILE_NAME}"), LINE_NUMBER.to_string()]
        )
    }

    #[test_case(Editor::Vim => format!("vim +{LINE_NUMBER} {FILE_NAME}"); "vim command")]
    #[test_case(Editor::Neovim => format!("nvim +{LINE_NUMBER} {FILE_NAME}"); "neovim command")]
    #[test_case(Editor::Nvim => format!("nvim +{LINE_NUMBER} {FILE_NAME}"); "nvim command")]
    #[test_case(Editor::Nano => format!("nano +{LINE_NUMBER} {FILE_NAME}"); "nano command")]
    #[test_case(Editor::Code => format!("code -g {FILE_NAME}:{LINE_NUMBER}"); "code command")]
    #[test_case(Editor::Vscode => format!("code -g {FILE_NAME}:{LINE_NUMBER}"); "vscode command")]
    #[test_case(Editor::CodeInsiders => format!("code-insiders -g {FILE_NAME}:{LINE_NUMBER}"); "code-insiders command")]
    #[test_case(Editor::Emacs => format!("emacs -nw +{LINE_NUMBER} {FILE_NAME}"); "emacs command")]
    #[test_case(Editor::Emacsclient => format!("emacsclient -nw +{LINE_NUMBER} {FILE_NAME}"); "emacsclient command")]
    #[test_case(Editor::Hx => format!("hx {FILE_NAME}:{LINE_NUMBER}"); "hx command")]
    #[test_case(Editor::Helix => format!("helix {FILE_NAME}:{LINE_NUMBER}"); "helix command")]
    #[test_case(Editor::Subl => format!("subl {FILE_NAME}:{LINE_NUMBER}"); "subl command")]
    #[test_case(Editor::SublimeText => format!("subl {FILE_NAME}:{LINE_NUMBER}"); "sublime text command")]
    #[test_case(Editor::Micro => format!("micro +{LINE_NUMBER} {FILE_NAME}"); "micro command")]
    #[test_case(Editor::Intellij => format!("idea --line {LINE_NUMBER} {FILE_NAME}"); "intellij command")]
    #[test_case(Editor::Goland => format!("goland --line {LINE_NUMBER} {FILE_NAME}"); "goland command")]
    #[test_case(Editor::Pycharm => format!("pycharm --line {LINE_NUMBER} {FILE_NAME}"); "pycharm command")]
    #[test_case(Editor::Less => format!("less +{LINE_NUMBER} {FILE_NAME}"); "less command")]
    fn builtin_editor_command(editor: Editor) -> String {
        let editor_command = EditorCommand::new(None, Some(editor)).unwrap();
        format!(
            "{} {}",
            editor_command.program(),
            editor_command.args(FILE_NAME, LINE_NUMBER).join(" ")
        )
    }
}
