complete ig --condition __fish_is_first_token --no-files
complete ig --long editor --description "Text editor used to open selected match" -xa "vim\t neovim\t nvim\t nano\t code\t vscode\t code-insiders\t emacs\t emacsclient\t hx\t helix\t subl\t sublime-text\t micro\t intellij\t goland\t pycharm\t less\t"
complete ig --long custom-command --description "Custom command used to open selected match" -xa ""
complete ig --long theme --description "UI color theme" -xa "light\t dark\t"
complete ig --long ignore-case --short i --description "Searches case insensitively" --condition "not __fish_seen_argument -- --ignore-case -i"
complete ig --long smart-case --short S --description "Enable smate case sensitivity" --condition "not __fish_seen_argument -- --smart-case -S"
complete ig --long hidden --short . --description "Searches hidden files and directories" --condition "not __fish_seen_argument -- --hidden -."
complete ig --long follow --short L --description "Follow symbolic links" --condition "not __fish_seen_argument -- --follow -L"
complete ig --long word-regexp --short w --description "Only show matches within word boundries" --condition "not __fish_seen_argument -- --word-regexp -w"
complete ig --long glob --short g --description "Search using given glob" -xa ""
complete ig --long type-list --description "Show all supported file types" --condition __fish_is_first_arg
complete ig --long type --short t --description "Only search files matching TYPE" -xa "(ig --type-list | string match -r '.*:' | string replace : '')"
complete ig --long type-not --short T --description "Only search files not matching TYPE" -xa "(ig --type-list | string match -r '.*:' | string replace : '')"
complete ig --long context-viewer --description "Context viewer position" -xa "vertical\t horizontal\t none\t " --keep
complete ig --long help --short h --description "Print help" --condition __fish_is_first_arg
complete ig --long version --short V --description "Print version" --condition __fish_is_first_arg
complete ig --long fixed-strings --short F --description "Exact matches with no regex" --condition "not __fish_seen_argument -- --fixed-strings -F"
complete ig --long sort --description "Sort results by" -xa "path\t modified\t accessed\t created\t"
complete ig --long sortr --description "Sort results reverse by" -xa "path\t modified\t accessed\t created\t"
