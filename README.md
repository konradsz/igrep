# ig - Interactive Grep
Runs [grep](https://crates.io/crates/grep) ([ripgrep's](https://github.com/BurntSushi/ripgrep/) library) in the background, allows interactively pick its results and open selected match in neovim (this is awfully hardcoded, but is subject to change).

<img src="./assets/demo.gif"/>

## Usage
`ig [FLAGS] <PATTERN> [PATH] [OPTIONS]`

### Flags
```
-h, --help           Prints help information
-i, --ignore-case    Searches case insensitively.
-S, --smart-case     Searches case insensitively if the pattern is all lowercase.
                     Search case sensitively otherwise.
-V, --version        Prints version information
```

### Options
```
-t, --type <TYPE>...     Only search files matching TYPE. Multiple type flags may be provided.
-T, --type-not <TYPE>... Do not search files matching TYPE. Multiple type-not flags may be provided.
```

### Keybindings
| Key                                            | Action                                         |
|------------------------------------------------|------------------------------------------------|
| `q`, `Esc`                                     | Quit                                           |
| `Down`, `j`                                    | Select next match                              |
| `Up`,`k`                                       | Select previous match                          |
| `Right`, `l`, `PageDown`                       | Select match in next file                      |
| `Left`, `h`, `PageUp`                          | Select match in previous file                  |
| `gg`, `Home`                                   | Jump to the first match                        |
| `Shift-g`, `End`                               | Jump to the last match                         |
| `Enter`                                        | Open current file                              |
| `dd`, `Delete`                                 | Filter out selected match                      |
| `dw`                                           | Filter out all matches in current file         |
| `F5`                                           | Re-run search                                  |
