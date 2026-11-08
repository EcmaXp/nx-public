set -g __atuin_history_seq 0

function _atuin_preexec --on-event fish_preexec
    set -e __atuin_id_file
    test -n "$fish_private_mode"; and return
    set -l dir /tmp
    set -q TMPDIR; and set dir $TMPDIR
    test -d "$RAMDIR"; and set dir $RAMDIR
    set __atuin_history_seq (math $__atuin_history_seq + 1)
    set -g __atuin_id_file $dir/atuin-$fish_pid-$__atuin_history_seq
    sh -c 'trap "" HUP; exec atuin history start -- "$1"' sh "$argv[1]" >$__atuin_id_file 2>/dev/null &
    disown
end

function _atuin_postexec --on-event fish_postexec --on-event fish_exit
    set -l s $status
    set -q __atuin_id_file; or return
    ATUIN_LOG=error sh -c '
        trap "" HUP
        i=0
        while [ ! -s "$1" ] && [ $i -lt 500 ]; do sleep 0.01; i=$((i + 1)); done
        id=$(cat "$1" 2>/dev/null) && [ -n "$id" ] && atuin history end --exit "$2" -- "$id"
        rm -f "$1"
    ' sh $__atuin_id_file $s &>/dev/null &
    disown
    set -e __atuin_id_file
end
