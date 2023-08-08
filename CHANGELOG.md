## v1.2.0 (2023-08-08)
***
- support multiple search paths
- Ctrl+c closes an application
- allow to change search pattern without closing an app

## v1.1.0 (2023-01-29)
***
- add error handling in case of editor process spawning failure
- improve performance by handling multiple file entries every redraw
- add support for Sublime Text, Micro, Intellij, Goland, Pycharm
- use `helix` as a binary name when `helix` is set as an editor of choice
- prefer $VISUAL variable over $EDITOR when determining text editor to use

## v1.0.0 (2023-01-08)
***
- add context viewer
- add support for opening files in Helix

## v0.5.1 (2022-08-01)
***
- add support for opening files in VS Code Insiders

## v0.5.0 (2022-04-24)
***
- add theme for light environments
- support for ripgrep's configuration file
- add Scoop package

## v0.4.0 (2022-03-16)
***
- improve clarity of using multi character input
- add support for opening files in VS Code
- add support for opening files in emacs and emacsclient

## v0.3.0 (2022-03-08)
***
- use $EDITOR as a fallback variable
- fix Initial console modes not set error on Windows
- make igrep available on Homebrew

## v0.2.0 (2022-03-02)
***
- allow to specify editor using `IGREP_EDITOR` environment variable
- add `nvim` as an alias for `neovim`
- support for searching hidden files/directories via `-.`/`--hidden` options

## v0.1.2 (2022-02-19)
***
Initial release. Provides basic set of functionalities.
