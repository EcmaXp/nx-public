function __direnv_should_export
    set -q DIRENV_DIR; and return 0
    set -l dir $PWD
    while not test -e $dir/.envrc
        test $dir = /; and return 1
        set dir (path dirname $dir)
    end
end
