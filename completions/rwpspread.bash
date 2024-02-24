_rwpspread() {
    local i cur prev opts cmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="rwpspread"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        rwpspread)
            opts="-i -a -b -d -p -s -f -h -V --image --align --backend --daemon --palette --swaylock --force-resplit --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --image)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --align)
                    COMPREPLY=($(compgen -W "tl tr tc bl br bc rc lc c" -- "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -W "tl tr tc bl br bc rc lc c" -- "${cur}"))
                    return 0
                    ;;
                --backend)
                    COMPREPLY=($(compgen -W "wpaperd swaybg" -- "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -W "wpaperd swaybg" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _rwpspread -o nosort -o bashdefault -o default rwpspread
else
    complete -F _rwpspread -o bashdefault -o default rwpspread
fi
