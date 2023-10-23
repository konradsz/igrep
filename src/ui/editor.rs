use crate::args::{EDITOR_ENV, IGREP_EDITOR_ENV, VISUAL_ENV};
use anyhow::{anyhow, Result};
use clap::ArgEnum;
use itertools::Itertools;
use std::{
    ffi::OsStr,
    fmt::{self, Debug, Display, Formatter},
    io,
    process::{Child, Command},
};
use strum_macros::Display;

#[derive(Display, Default, PartialEq, Eq, Copy, Clone, Debug, ArgEnum)]
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

impl Editor {
    pub fn determine(editor_cli: Option<Editor>) -> Result<Editor> {
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
                Editor::from_str(&Editor::extract_editor_name(&value), false)
                    .map_err(|error| add_error_context(error, value, name))
            })
        };

        editor_cli
            .map(Ok)
            .or_else(|| read_from_env(IGREP_EDITOR_ENV))
            .or_else(|| read_from_env(VISUAL_ENV))
            .or_else(|| read_from_env(EDITOR_ENV))
            .unwrap_or(Ok(Editor::default()))
    }

    pub fn spawn(self, file_name: &str, line_number: u64) -> io::Result<Child> {
        let mut command = EditorCommand::new(self, file_name, line_number);

        command.spawn()
    }

    fn extract_editor_name(input: &str) -> String {
        let mut split = input.rsplit('/');
        split.next().unwrap().into()
    }
}

struct EditorCommand(Command);

impl EditorCommand {
    fn new(editor: Editor, file_name: &str, line_number: u64) -> Self {
        let mut command = Command::new(Self::program(editor));
        command.args(Self::args(editor, file_name, line_number));
        Self(command)
    }

    fn program(editor: Editor) -> String {
        match editor {
            Editor::Vim => "vim".into(),
            Editor::Neovim | Editor::Nvim => "nvim".into(),
            Editor::Nano => "nano".into(),
            Editor::Code | Editor::Vscode => "code".into(),
            Editor::CodeInsiders => "code-insiders".into(),
            Editor::Emacs => "emacs".into(),
            Editor::Emacsclient => "emacsclient".into(),
            Editor::Hx => "hx".into(),
            Editor::Helix => "helix".into(),
            Editor::Subl | Editor::SublimeText => "subl".into(),
            Editor::Micro => "micro".into(),
            Editor::Intellij => "idea".into(),
            Editor::Goland => "goland".into(),
            Editor::Pycharm => "pycharm".into(),
            Editor::Less => "less".into(),
        }
    }

    fn args(editor: Editor, file_name: &str, line_number: u64) -> Box<dyn Iterator<Item = String>> {
        match editor {
            Editor::Vim
            | Editor::Neovim
            | Editor::Nvim
            | Editor::Nano
            | Editor::Micro
            | Editor::Less => Box::new([format!("+{line_number}"), file_name.into()].into_iter()),
            Editor::Code | Editor::Vscode | Editor::CodeInsiders => {
                Box::new(["-g".into(), format!("{file_name}:{line_number}")].into_iter())
            }
            Editor::Emacs | Editor::Emacsclient => {
                Box::new(["-nw".into(), format!("+{line_number}"), file_name.into()].into_iter())
            }
            Editor::Hx | Editor::Helix | Editor::Subl | Editor::SublimeText => {
                Box::new([format!("{file_name}:{line_number}")].into_iter())
            }
            Editor::Intellij | Editor::Goland | Editor::Pycharm => {
                Box::new(["--line".into(), format!("{line_number}"), file_name.into()].into_iter())
            }
        }
    }

    fn spawn(&mut self) -> io::Result<Child> {
        self.0.spawn()
    }
}

impl Display for EditorCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.0.get_program().to_string_lossy(),
            self.0.get_args().map(OsStr::to_string_lossy).join(" ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::EditorOpt;
    use clap::Parser;
    use lazy_static::lazy_static;
    use test_case::test_case;

    lazy_static! {
        static ref SERIAL_TEST: std::sync::Mutex<()> = Default::default();
    }

    #[test_case(Some("nano"), Some("vim"), None, Some("neovim") => matches Ok(Editor::Nano); "cli")]
    #[test_case(None, Some("nano"), None, Some("neovim") => matches Ok(Editor::Nano); "igrep env")]
    #[test_case(None, None, Some("nano"), Some("helix") => matches Ok(Editor::Nano); "visual env")]
    #[test_case(None, None, None, Some("nano") => matches Ok(Editor::Nano); "editor env")]
    #[test_case(Some("unsupported-editor"), None, None, None => matches Err(_); "unsupported cli")]
    #[test_case(None, Some("unsupported-editor"), None, None => matches Err(_); "unsupported igrep env")]
    #[test_case(None, None, None, Some("unsupported-editor") => matches Err(_); "unsupported editor env")]
    #[test_case(None, None, None, None => matches Ok(Editor::Vim); "default editor")]
    #[test_case(None, Some("/usr/bin/nano"), None, None => matches Ok(Editor::Nano); "igrep env path")]
    #[test_case(None, None, None, Some("/usr/bin/nano") => matches Ok(Editor::Nano); "editor env path")]
    fn editor_options_precedence(
        cli_option: Option<&str>,
        igrep_editor_env: Option<&str>,
        visual_env: Option<&str>,
        editor_env: Option<&str>,
    ) -> Result<Editor> {
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

        Editor::determine(opt?.editor)
    }

    const FILE_NAME: &str = "file_name";
    const LINE_NUMBER: u64 = 123;

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
    fn editor_command(editor: Editor) -> String {
        EditorCommand::new(editor, FILE_NAME, LINE_NUMBER).to_string()
    }
}
