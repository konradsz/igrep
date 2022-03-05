use clap::ArgEnum;
use std::fmt::Display;

#[derive(PartialEq, Copy, Clone, Debug, ArgEnum)]
pub enum Editor {
    Vim,
    Neovim,
    Nvim,
    Nano,
}

impl Default for Editor {
    fn default() -> Self {
        Self::Vim
    }
}

impl Display for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = match self {
            Editor::Vim => "vim",
            Editor::Neovim | Editor::Nvim => "nvim",
            Editor::Nano => "nano",
        };

        write!(f, "{}", command)
    }
}
