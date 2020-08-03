# ig - Interactive Grep
Runs ripgrep in the background, allows interactively pick its results and open selected match in vim (this may be subject to change).

## Usage
`ig [FLAGS] <PATTERN> [PATH]`

### Flags
```
-h, --help           Prints help information
-i, --ignore-case    Searches case insensitively.
-S, --smart-case     Searches case insensitively if the pattern is all lowercase.
                     Search case sensitively otherwise.
-V, --version        Prints version information
```

### Keybindings
|                                                |                                                |
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
