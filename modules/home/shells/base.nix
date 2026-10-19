{ config, lib, ... }:
let
  sessionPath = config.home.sessionPath;
in
{
  imports = [
    ./aliases/base.nix
  ];

  programs.bash = {
    bashrcExtra = ''
      . "${config.home.sessionVariablesPackage}/etc/profile.d/hm-session-vars.sh"
    '';
  };

  programs.fish = {
    enable = true;
    interactiveShellInit = ''
      set -g fish_greeting
    '';
    functions = {
      "fish_config" = {
        body = ''
          if test "$argv" = "theme choose default --no-override"
              and test -f $__fish_config_dir/conf.d/fish_frozen_theme.fish
              return
          end
          source $__fish_data_dir/functions/fish_config.fish
          fish_config $argv
        '';
      };
    };
  };

  programs.nushell = {
    extraEnv = lib.mkIf (sessionPath != [ ]) ''
      if ($env.__HM_SESS_VARS_SOURCED? | is-empty) {
        $env.PATH = (${builtins.toJSON sessionPath} ++ $env.PATH)
      }
    '';

    envFile.text = ''
      $env.config = {
        show_banner: false,
      }
    '';
  };
}
