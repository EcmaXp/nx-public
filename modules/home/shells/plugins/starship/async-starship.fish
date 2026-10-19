set -g VIRTUAL_ENV_DISABLE_PROMPT 1
builtin functions -e fish_mode_prompt
set -gx STARSHIP_SHELL fish
set -gx STARSHIP_SESSION_KEY (string sub -s1 -l16 (random)(random)(random)(random)(random)0000000000000000)

if test -z __ASYNC_STARSHIP_DISABLED
    return
end

set __ASYNC_STARSHIP_TMPDIR $TMPDIR
if test -d $RAMDIR
    set __ASYNC_STARSHIP_TMPDIR $RAMDIR
end

set __ASYNC_STARSHIP_PROMPT_DIR "$__ASYNC_STARSHIP_TMPDIR/async-starship"
set __ASYNC_STARSHIP_LEFT_PROMPT_FILE "$__ASYNC_STARSHIP_PROMPT_DIR/$STARSHIP_SESSION_KEY-left.prompt"
set __ASYNC_STARSHIP_CACHE_DIR "$__ASYNC_STARSHIP_PROMPT_DIR/cache"
set __ASYNC_STARSHIP_LAST_PWD $PWD
set __ASYNC_STARSHIP_SPAWNED_PWD
set __ASYNC_STARSHIP_PROMPT_LINES ""
set __ASYNC_STARSHIP_PROMPT ""

function __async_starship_read_prompt
    while read -l line
        echo $line
    end < $argv[1]
end

function __async_starship_cache_file
    set -l name (string escape --style=var -- $PWD)
    test (string length -- $name) -le 200; and echo $__ASYNC_STARSHIP_CACHE_DIR/$name
end

function __async_starship_init
    test -d $__ASYNC_STARSHIP_CACHE_DIR; or mkdir -p $__ASYNC_STARSHIP_CACHE_DIR
    set -l cache (__async_starship_cache_file)
    if test -n "$cache" -a -f "$cache"
        set __ASYNC_STARSHIP_PROMPT_LINES (__async_starship_read_prompt $cache)
        string join \n $__ASYNC_STARSHIP_PROMPT_LINES | read -z __ASYNC_STARSHIP_PROMPT
        return
    end
    set __ASYNC_STARSHIP_PROMPT_LINES "" (set_color --bold white)fish(set_color normal)" " ""
    __async_starship_ready_prompt_new_directory
end

function __async_starship_ready_prompt_default
    set __ASYNC_STARSHIP_PROMPT_LINES (__async_starship_read_prompt $__ASYNC_STARSHIP_LEFT_PROMPT_FILE)
    string join \n $__ASYNC_STARSHIP_PROMPT_LINES | read -z __ASYNC_STARSHIP_PROMPT
end

function __async_starship_placeholder_pwd
    set -l root $PWD
    while not test -e $root/.git
        if test $root = /
            string replace -r -- '^'(string escape --style=regex -- $HOME)'(?=/|$)' '~' $PWD
            return
        end
        set root (path dirname $root)
    end
    string replace -- (path dirname $root)/ '…/' $PWD
end

function __async_starship_ready_prompt_new_directory
    set -l dir (set_color --bold cyan)(__async_starship_placeholder_pwd)(set_color normal)
    test -w $PWD; or set dir $dir(set_color red)🔒(set_color normal)
    set __ASYNC_STARSHIP_PROMPT_LINES $__ASYNC_STARSHIP_PROMPT_LINES[..-2] "$dir "(set_color --bold green)"❯"(set_color normal)" "
    string join \n $__ASYNC_STARSHIP_PROMPT_LINES | read -z __ASYNC_STARSHIP_PROMPT
end

function __async_starship_on_prompt --on-event fish_prompt
    set -l last_pipestatus $pipestatus
    set -l last_status $status
    if set -q __ASYNC_STARSHIP_BOOT_STATE
        set -l boot $__ASYNC_STARSHIP_BOOT_STATE
        set -e __ASYNC_STARSHIP_BOOT_STATE
        test "$boot" = "$PWD $DIRENV_DIFF"; and return
    end
    __async_starship_render $last_status $last_pipestatus
end

function __async_starship_render
    set __ASYNC_STARSHIP_SPAWNED_PWD $PWD
    switch "$fish_key_bindings"
        case fish_hybrid_key_bindings fish_vi_key_bindings
            set STARSHIP_KEYMAP "$fish_bind_mode"
        case '*'
            set STARSHIP_KEYMAP insert
    end
    set STARSHIP_CMD_STATUS $argv[1]
    set STARSHIP_CMD_PIPESTATUS $argv[2..]
    set STARSHIP_DURATION "$CMD_DURATION$cmd_duration"
    set STARSHIP_JOBS (count (jobs -p))
    env \
        SHLVL=(math $SHLVL - 1) \
        COLUMNS=$COLUMNS \
        STARSHIP_KEYMAP=$STARSHIP_KEYMAP \
        STARSHIP_CMD_PIPESTATUS=$STARSHIP_CMD_PIPESTATUS \
        STARSHIP_CMD_STATUS=$STARSHIP_CMD_STATUS \
        STARSHIP_DURATION=$STARSHIP_DURATION \
        STARSHIP_JOBS=$STARSHIP_JOBS \
        __ASYNC_STARSHIP_PROMPT_DIR=$__ASYNC_STARSHIP_PROMPT_DIR \
        __ASYNC_STARSHIP_LEFT_PROMPT_FILE=$__ASYNC_STARSHIP_LEFT_PROMPT_FILE \
        __ASYNC_STARSHIP_CACHE_FILE=(__async_starship_cache_file) \
    sh -c '
        mkdir $__ASYNC_STARSHIP_PROMPT_DIR 2>/dev/null
        __ASYNC_STARSHIP_LEFT_RANDOM_PROMPT_FILE=$__ASYNC_STARSHIP_LEFT_PROMPT_FILE.$RANDOM-tmp.prompt
        starship prompt \
            --terminal-width=$COLUMNS \
            --status=$STARSHIP_CMD_STATUS \
            --pipestatus="$STARSHIP_CMD_PIPESTATUS" \
            --keymap=$STARSHIP_KEYMAP \
            --cmd-duration=$STARSHIP_DURATION \
            --jobs=$STARSHIP_JOBS \
            > $__ASYNC_STARSHIP_LEFT_RANDOM_PROMPT_FILE
        if kill -0 $PPID 2>/dev/null; then
            mv -f $__ASYNC_STARSHIP_LEFT_RANDOM_PROMPT_FILE $__ASYNC_STARSHIP_LEFT_PROMPT_FILE
            kill -s USR1 $PPID
        else
            rm -f $__ASYNC_STARSHIP_LEFT_RANDOM_PROMPT_FILE
            exit
        fi
        if [ -n "$__ASYNC_STARSHIP_CACHE_FILE" ]; then
            mkdir -p "${__ASYNC_STARSHIP_CACHE_FILE%/*}"
            SHLVL=1 starship prompt \
                --terminal-width=$COLUMNS \
                --status=0 \
                --pipestatus=0 \
                --keymap=insert \
                --cmd-duration=0 \
                --jobs=0 \
                > "$__ASYNC_STARSHIP_CACHE_FILE.$$.tmp"
            mv -f "$__ASYNC_STARSHIP_CACHE_FILE.$$.tmp" "$__ASYNC_STARSHIP_CACHE_FILE"
        fi
    ' &
    disown $last_pid
end

function __async_starship_signal_handler --on-signal SIGUSR1
    __async_starship_ready_prompt_default
    commandline -f repaint
end

function __async_starship_cleanup --on-event fish_exit
    command rm -f $__ASYNC_STARSHIP_LEFT_PROMPT_FILE
end

__async_starship_init
set -g __ASYNC_STARSHIP_BOOT_STATE "$PWD" "$DIRENV_DIFF"
__async_starship_render 0 0

function fish_prompt
    set -l last_pipestatus $pipestatus
    set -l last_status $status
    if test $PWD != $__ASYNC_STARSHIP_LAST_PWD
        __async_starship_ready_prompt_new_directory
    end

    echo -n $__ASYNC_STARSHIP_PROMPT

    if test $PWD != $__ASYNC_STARSHIP_LAST_PWD
        set __ASYNC_STARSHIP_LAST_PWD $PWD
        if test "$__ASYNC_STARSHIP_SPAWNED_PWD" != $PWD
            __async_starship_render $last_status $last_pipestatus
        end
    end
end

function fish_right_prompt
    # TODO: Implement right prompt
end
