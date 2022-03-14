use crate::args::{EDITOR_ENV, IGREP_EDITOR_ENV};
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

#[derive(Display, PartialEq, Copy, Clone, Debug, ArgEnum)]
#[strum(serialize_all = "lowercase")]
pub enum Editor {
    Vim,
    Neovim,
    Nvim,
    Nano,
    Code,
    Vscode,
}

impl Default for Editor {
    fn default() -> Self {
        Self::Vim
    }
}

impl Editor {
    pub fn determine(editor_cli: Option<Editor>) -> Result<Editor> {
        let add_error_context = |e: String, env_value: String, env_name: &str| {
            let possible_variants = Editor::value_variants()
                .iter()
                .map(Editor::to_string)
                .join(", ");
            anyhow!(e).context(format!(
                "\"{}\" read from ${}, possible variants: [{}]",
                env_value, env_name, possible_variants
            ))
        };

        if let Some(editor_cli) = editor_cli {
            Ok(editor_cli)
        } else if let Ok(value) = std::env::var(IGREP_EDITOR_ENV) {
            Editor::from_str(&value, false)
                .map_err(|error| add_error_context(error, value, IGREP_EDITOR_ENV))
        } else if let Ok(value) = std::env::var(EDITOR_ENV) {
            Editor::from_str(&value, false)
                .map_err(|error| add_error_context(error, value, EDITOR_ENV))
        } else {
            Ok(Editor::default())
        }
    }

    pub fn spawn(self, file_name: &str, line_number: u64) -> Child {
        let mut command = EditorCommand::new(self, file_name, line_number);
        command.spawn().unwrap_or_else(|_| {
            panic!(
                "Error: Failed to run editor with a command: \"{}\"",
                command
            );
        })
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
        }
    }

    fn args(editor: Editor, file_name: &str, line_number: u64) -> impl IntoIterator<Item = String> {
        match editor {
            Editor::Vim | Editor::Neovim | Editor::Nvim | Editor::Nano => {
                [format!("+{line_number}"), file_name.into()]
            }
            Editor::Code | Editor::Vscode => ["-g".into(), format!("{file_name}:{line_number}")],
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

    #[test_case(Some("nano"), Some("vim"), Some("neovim") => matches Ok(Editor::Nano); "cli")]
    #[test_case(None, Some("nano"), Some("neovim") => matches Ok(Editor::Nano); "igrep env")]
    #[test_case(None, None, Some("nano") => matches Ok(Editor::Nano); "editor env")]
    #[test_case(Some("unsupported-editor"), None, None => matches Err(_); "unsupported cli")]
    #[test_case(None, Some("unsupported-editor"), None => matches Err(_); "unsupported igrep env")]
    #[test_case(None, None, Some("unsupported-editor") => matches Err(_); "unsupported editor env")]
    #[test_case(None, None, None => matches Ok(Editor::Vim); "default editor")]
    fn editor_options_precedence(
        cli_option: Option<&str>,
        igrep_editor_env: Option<&str>,
        editor_env: Option<&str>,
    ) -> Result<Editor> {
        let _guard = SERIAL_TEST.lock().unwrap();
        std::env::remove_var(IGREP_EDITOR_ENV);
        std::env::remove_var(EDITOR_ENV);

        let opt = if let Some(cli_option) = cli_option {
            EditorOpt::try_parse_from(&["test", "--editor", cli_option])
        } else {
            EditorOpt::try_parse_from(&["test"])
        };

        if let Some(igrep_editor_env) = igrep_editor_env {
            std::env::set_var(IGREP_EDITOR_ENV, igrep_editor_env);
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
    fn editor_command(editor: Editor) -> String {
        EditorCommand::new(editor, FILE_NAME, LINE_NUMBER).to_string()
    }
}
