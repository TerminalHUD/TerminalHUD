# Build a trie of the existing ESC keymap before installing a shorter binding.
# Readline's shorter-binding fallback otherwise runs for unknown longer keys too.
_terminalhud_guard_prefixes() {
    local LC_ALL=C line key char token value prefix edge i j rest oct
    local -A prefixes=() edges=() bindings=()
    prefixes['\033']=1
    while IFS= read -r line; do
        [[ $line =~ ^\"(([^\"\\]|\\.)*)\"[:[:space:]] ]] || continue
        key=${BASH_REMATCH[1]}
        [[ $key == '\e'* ]] || continue
        prefix=
        i=0
        while (( i < ${#key} )); do
            char=${key:i:1}; ((i+=1))
            if [[ $char != '\' ]]; then
                printf -v value '%d' "'$char"
            else
                token=${key:i:1}; ((i+=1))
                case $token in
                    e) value=27 ;;
                    C)
                        [[ ${key:i:1} == '-' ]] || return 1
                        ((i+=1)); char=${key:i:1}; ((i+=1))
                        printf -v value '%d' "'$char"
                        if [[ $char == '?' ]]; then value=127; else ((value &= 31)); fi ;;
                    [0-7])
                        rest=$token
                        for ((j=0; j<2; j++)); do
                            [[ ${key:i:1} == [0-7] ]] || break
                            rest+=${key:i:1}; ((i+=1))
                        done
                        value=$((8#$rest)) ;;
                    '\'|'"') printf -v value '%d' "'$token" ;;
                    *) return 1 ;;
                esac
            fi
            printf -v oct '\\%03o' "$value"
            if [[ -n $prefix ]]; then
                prefixes[$prefix]=1
                edges["$prefix/$oct"]=1
            fi
            prefix+=$oct
        done
        bindings[$prefix]=1
    done < <(bind -m emacs-standard -p; bind -m emacs-standard -s; bind -m emacs-standard -X)
    for prefix in "${!prefixes[@]}"; do
        # A user-defined shorter fallback has behavior we cannot replace.
        [[ ${bindings[$prefix]-} != 1 ]] || return 1
    done
    # Undefined keys retain their normal abort behavior rather than falling back
    # to the new standalone Escape handler and replaying their suffix as text.
    for prefix in "${!prefixes[@]}"; do
        for ((i=0; i<256; i++)); do
            printf -v oct '\\%03o' "$i"
            edge=$prefix/$oct
            [[ ${edges[$edge]-} == 1 ]] || bind -m emacs-standard "\"$prefix$oct\": abort"
        done
        # An incomplete longer prefix must not fall back to standalone Escape.
        [[ $prefix == '\033' ]] || bind -m emacs-standard "\"$prefix\": abort"
    done
}
