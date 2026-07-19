# Completions for nx command

# Get available nx-* subcommands
function __nx_subcommands
    complete -C 'nx-' | string split -f1 \t | string replace 'nx-' ''
end

# Get completions for a specific nx-* command (just-based)
function __nx_subcommand_recipes
    set -l cmd $argv[1]
    nx-$cmd --summary 2>/dev/null | string split ' '
end

# Check if we already have a subcommand
function __nx_needs_subcommand
    set -l tokens (commandline -opc)
    test (count $tokens) -eq 1
end

# Get the current subcommand
function __nx_current_subcommand
    set -l tokens (commandline -opc)
    if test (count $tokens) -ge 2
        echo $tokens[2]
    end
end

# Complete subcommands (clean, repl, sync, refresh, orbstack, etc.)
complete -c nx -f -n __nx_needs_subcommand -a '(__nx_subcommands)'

# Complete recipes for each subcommand
complete -c nx -f -n 'not __nx_needs_subcommand' -a '(__nx_subcommand_recipes (__nx_current_subcommand))'
