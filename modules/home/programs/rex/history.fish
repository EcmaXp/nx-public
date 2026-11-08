if set -q REX_BLOCK; and not set -q CLAUDECODE
    set -l rex_id (string replace block: "" -- $REX_BLOCK)
    set -l history_dir $HOME/.local/state/nx/rex-history
    if status is-login; and test "$NX_REX_BLOCK_SHELL" != "$REX_BLOCK"
        if test -f $HOME/.cache/nx/agent/rex-blocks/$rex_id.json -o -f $history_dir/$rex_id.json
            function __nx_rex_restore --on-event fish_prompt
                functions --erase __nx_rex_restore
                nx agents restore-block $REX_BLOCK
            end
        end
    end
    set -gx NX_REX_BLOCK_SHELL $REX_BLOCK
    test -d $history_dir; or command mkdir -p $history_dir
    set -g __nx_rex_returned $history_dir/$rex_id.returned

    function __nx_rex_follow --on-event fish_preexec
        string match -qr '^\s*claude(\s|$)' -- $argv[1]; and return
        sh -c "trap 'kill \$s; exit' TERM; sleep 45 & s=\$!; wait \$s && exec nx agents save-block $REX_BLOCK --follow $fish_pid" >/dev/null 2>&1 &
        set -g __nx_rex_follower $last_pid
        disown $last_pid 2>/dev/null
    end

    function __nx_rex_save --on-event fish_postexec
        set -l code $status
        if set -q __nx_rex_follower
            command kill $__nx_rex_follower 2>/dev/null
            set -e __nx_rex_follower
        end
        test $code -eq 129; and return
        LC_ALL=C jobs | string match -qr '\tstopped\t'; or true >$__nx_rex_returned
        nx agents save-block $REX_BLOCK >/dev/null 2>&1 &
        disown $last_pid 2>/dev/null
    end
end
