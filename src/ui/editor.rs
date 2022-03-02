use std::fmt::Display;

use clap::ArgEnum;

#[derive(Copy, Clone, Debug, ArgEnum)]
pub enum Editor {
    Vim,
    Neovim,
    Nano,
}

impl Display for Editor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let command = match self {
            Editor::Vim => "vim",
            Editor::Neovim => "nvim",
            Editor::Nano => "nano",
            Editor::Micro => 
        };

        write!(f, "{}", command)
    }
}
