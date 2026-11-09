# TerminalHUD prompt-aware Escape. Source only from an interactive Bash startup file.
# Readline keeps longer ESC sequences in its existing prefix keymap when bind -x
# adds the shorter binding; its keyseq-timeout resolves standalone Escape.
[[ $- == *i* ]] || return 0
[[ ${_terminalhud_escape_loaded-} != 1 ]] || return 0

_terminalhud_enable_escape() {
    local setting binding
    if (( BASH_VERSINFO[0] < 5 )) || ! [[ -o emacs ]] || ! shopt -q promptvars; then
        printf '%s\n' 'TerminalHUD: Escape requires Bash 5+, Emacs editing and promptvars; bindings unchanged.' >&2
        return 1
    fi
    setting=$(bind -v)
    if [[ $setting != *'set keyseq-timeout '* ]] ||
        [[ $setting == *'set keyseq-timeout 0'* || $setting == *'set keyseq-timeout -'* ]]; then
        printf '%s\n' 'TerminalHUD: Escape requires a positive Readline keyseq-timeout; bindings unchanged.' >&2
        return 1
    fi
    # Never replace an existing standalone Escape function, macro or shell command.
    while IFS= read -r binding; do
        if [[ $binding == '"\e":'* || $binding == '"\e" '* ]]; then
            printf '%s\n' 'TerminalHUD: custom Escape binding detected; bindings unchanged.' >&2
            return 1
        fi
    done < <(bind -p; bind -s; bind -X)
    source "$_terminalhud_source_dir/prefixes.bash"
    if ! _terminalhud_guard_prefixes; then
        printf '%s\n' 'TerminalHUD: unsupported custom Escape prefix map; bindings unchanged.' >&2
        return 1
    fi
    _terminalhud_ps1='${_terminalhud_prompt:$((_terminalhud_primary=1)):0}'
    _terminalhud_ps2='${_terminalhud_prompt:$((_terminalhud_primary=0)):0}'
    _terminalhud_prompt=
    PS1=$_terminalhud_ps1${PS1-}
    PS2=$_terminalhud_ps2${PS2-}
    bind -m emacs-standard -x '"\e":_terminalhud_escape'
    _terminalhud_escape_loaded=1
}

_terminalhud_escape() {
    # These sentinels distinguish PS2, where READLINE_LINE can also be empty.
    shopt -q promptvars || return 0
    [[ -o emacs && ${_terminalhud_primary-0} == 1 && -z $READLINE_LINE &&
       ${PS1-} == "$_terminalhud_ps1"* && ${PS2-} == "$_terminalhud_ps2"* ]] || return 0
    # A shell-owned background child keeps even an unresponsive IPC owner from
    # blocking editing. The client only contacts the existing same-user owner.
    [[ -f $_terminalhud_source_dir/../powershell/Request-Passive.ps1 ]] || return 0
    {
        ( powershell.exe -NoLogo -NoProfile -NonInteractive -File "$_terminalhud_client" ) &
        disown "$!" || :
    } >/dev/null 2>&1
}

# The source file and client are kept together in the receipted version payload.
_terminalhud_source_dir=${BASH_SOURCE[0]%/*}
if command -v wslpath >/dev/null 2>&1; then
    _terminalhud_client=$(wslpath -w "$_terminalhud_source_dir/../powershell/Request-Passive.ps1")
elif command -v cygpath >/dev/null 2>&1; then
    _terminalhud_client=$(cygpath -w "$_terminalhud_source_dir/../powershell/Request-Passive.ps1")
else
    # Also permits owned Linux PTY tests with a fake powershell.exe.
    _terminalhud_client=$_terminalhud_source_dir/../powershell/Request-Passive.ps1
fi
_terminalhud_enable_escape || :
