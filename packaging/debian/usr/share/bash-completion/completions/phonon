# bash completion for phonon -*- shell-script -*-

_phonon() {
    local cur prev words cword
    if declare -F _init_completion >/dev/null 2>&1; then
        _init_completion || return
    else
        COMPREPLY=()
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=("${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    fi

    local commands="ui gui validate run sweep mc help"
    local formats="csv json binary"

    if [[ $cword -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$commands -h --help -V --version" -- "$cur") )
        return 0
    fi

    case "${words[1]}" in
        ui|gui)
            COMPREPLY=()
            ;;
        validate)
            COMPREPLY=( $(compgen -f -- "$cur") )
            ;;
        run)
            case "$prev" in
                -f|--format)
                    COMPREPLY=( $(compgen -W "$formats" -- "$cur") )
                    ;;
                -o|--output)
                    COMPREPLY=( $(compgen -f -- "$cur") )
                    ;;
                *)
                    COMPREPLY=( $(compgen -f -- "$cur") )
                    ;;
            esac
            ;;
        sweep)
            case "$prev" in
                -f|--format)
                    COMPREPLY=( $(compgen -W "$formats" -- "$cur") )
                    ;;
                -o|--output)
                    COMPREPLY=( $(compgen -f -- "$cur") )
                    ;;
                *)
                    COMPREPLY=( $(compgen -f -- "$cur") )
                    ;;
            esac
            ;;
        mc)
            case "$prev" in
                -f|--format)
                    COMPREPLY=( $(compgen -W "$formats" -- "$cur") )
                    ;;
                -o|--output)
                    COMPREPLY=( $(compgen -f -- "$cur") )
                    ;;
                *)
                    COMPREPLY=( $(compgen -f -- "$cur") )
                    ;;
            esac
            ;;
    esac
}

complete -F _phonon phonon
