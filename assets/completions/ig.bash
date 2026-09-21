#!/usr/bin/env bash

_ig()
{
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    opts="
        --help -h
        --version -V
        --editor
        --custom-command
        --theme
        --type -t
        --type-not -T
        --context-viewer
        --sort
        --sortr
        --glob
        --type-list
        --ignore-case -i
        --smart-case -S
        --hidden -.
        --follow -L
        --word-regexp -w
        --fixed-strings -F
    "

    case "${prev}" in
        --editor)
            COMPREPLY=( $(compgen -W "vim neovim nvim nano code vscode code-insiders emacs emacsclient hx helix subl sublime-text micro intellij goland pycharm less" -- "$cur") )
            return 0
            ;;
        --custom-command)
            return 0
            ;;
        --theme)
            COMPREPLY=( $(compgen -W "light dark" -- "$cur") )
            return 0
            ;;
        --type)
            local types=$(ig --type-list 2>/dev/null | grep -oE '^[^:]+' | tr '\n' ' ')
            COMPREPLY=( $(compgen -W "${types}" -- "$cur") )
            return 0
            ;;
        --type-not)
            local types=$(ig --type-list 2>/dev/null | grep -oE '^[^:]+' | tr '\n' ' ')
            COMPREPLY=( $(compgen -W "${types}" -- "$cur") )
            return 0
            ;;
        --context-viewer)
            COMPREPLY=( $(compgen -W "vertical horizontal none" -- "$cur") )
            return 0
            ;;
        --sort)
            COMPREPLY=( $(compgen -W "path modified accessed created" -- "$cur") )
            return 0
            ;;
        --sortr)
            COMPREPLY=( $(compgen -W "path modified accessed created" -- "$cur") )
            return 0
            ;;
        --glob)
            return 0
            ;;
        *)
            ;;
    esac

    case "$2" in
    -*)
        mapfile -t COMPREPLY < <(compgen -W "$opts" -- """$2""")
        return 0
        ;;
    *)
        if (( COMP_CWORD > 1 )); then
            mapfile -t COMPREPLY < <(compgen -fd -- """$2""")
            return 0
        fi
        ;;
    esac

}

complete -o filenames -F _ig ig
