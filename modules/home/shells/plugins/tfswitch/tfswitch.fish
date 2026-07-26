set __TFSWITCH_LAST_ACTIVATED_HASH ""

function __tfswitch
    if count *.tf > /dev/null
        set --local __TFSWITCH_HASH "$(grep -H "required_version" *.tf | sha256sum)"
        if test $__TFSWITCH_HASH != "$__TFSWITCH_LAST_ACTIVATED_HASH"
            set __TFSWITCH_LAST_ACTIVATED_HASH $__TFSWITCH_HASH
            set --local __TFSWITCH_TERRAFORM_VERSION (__tfswitch_get_terraform_version)
            set --local __TFSWITCH_MISE_TERRAFORM_VERSION (__tfswitch_get_mise_terraform_version)
            if test "$__TFSWITCH_TERRAFORM_VERSION" != "$__TFSWITCH_MISE_TERRAFORM_VERSION"
                __tfswitch_mise_terraform use $__TFSWITCH_TERRAFORM_VERSION
            end
        end
    end
end

function __tfswitch_mise_terraform
    set --local MISE_COMMAND $argv[1]
    set --local MISE_TERRAFORM_VERSION $argv[2]
    set --local MISE_ARGS $argv[3..]
    if string match --regex "^0" $MISE_TERRAFORM_VERSION > /dev/null
        ASDF_HASHICORP_OVERWRITE_ARCH=amd64 mise $MISE_COMMAND "terraform@$MISE_TERRAFORM_VERSION" $MISE_ARGS
    else
        mise $MISE_COMMAND "terraform@$MISE_TERRAFORM_VERSION" $MISE_ARGS
    end
end

function __tfswitch_get_terraform_version
    echo | command tfswitch --dry-run 2>&1 | string match --regex --groups-only 'Matched version: "(.*?)"'
end

function __tfswitch_get_mise_terraform_version
    tomlq -r .tools.terraform .mise.toml 2>/dev/null
end

function tfswitch
    if test -z $argv
        set --local __TFSWITCH_TERRAFORM_VERSION (__tfswitch_get_terraform_version)
        if test -n "$__TFSWITCH_TERRAFORM_VERSION"
            __tfswitch_mise_terraform use $__TFSWITCH_TERRAFORM_VERSION
        else
            echo "No terraform version found in the current directory" >&2
            return 1
        end
    end

    command tfswitch $argv
end

function terraform
    __tfswitch
    set --local __TFSWITCH_MISE_TERRAFORM_VERSION (__tfswitch_get_mise_terraform_version)
    if test -n "$__TFSWITCH_MISE_TERRAFORM_VERSION"
        __tfswitch_mise_terraform exec $__TFSWITCH_MISE_TERRAFORM_VERSION -- terraform $argv
    else
        command terraform $argv
    end
end
