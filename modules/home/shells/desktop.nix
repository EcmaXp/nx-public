{
  config,
  lib,
  osConfig,
  pkgs,
  ...
}:
let
  homeDir = config.home.homeDirectory;
  brewPrefix = "/opt/homebrew";
  nixProfilePaths = map (
    profile:
    lib.replaceStrings
      [ "$HOME" "$USER" "\${XDG_STATE_HOME}" ]
      [ homeDir config.home.username config.xdg.stateHome ]
      "${profile}/bin"
  ) osConfig.environment.profiles;
in
{
  imports = [
    ./aliases/desktop.nix
  ];

  config = lib.mkIf config.nx.home.shells.desktop.enable {
    home.sessionPath = lib.mkMerge [
      [
        "${homeDir}/.local/bin"
        "${homeDir}/go/bin"
        "${homeDir}/.cargo/bin"
        "${homeDir}/.krew/bin"
        "${homeDir}/.dotnet/tools"
      ]
      (lib.mkAfter (
        nixProfilePaths
        ++ lib.optionals pkgs.stdenv.isDarwin [
          "${brewPrefix}/bin"
          "${brewPrefix}/sbin"
        ]
      ))
    ];

    home.sessionVariables = lib.mkIf pkgs.stdenv.isDarwin {
      HOMEBREW_PREFIX = brewPrefix;
      HOMEBREW_CELLAR = "${brewPrefix}/Cellar";
      HOMEBREW_REPOSITORY = brewPrefix;
      INFOPATH = "${brewPrefix}/share/info:$INFOPATH";
    };

    home.sessionSearchVariables = lib.mkIf pkgs.stdenv.isDarwin {
      MANPATH = [ "" ];
    };

    programs.fish = {
      functions = {
        # nix-command-not-found
        __fish_command_not_found_handler = {
          body = "~/.local/bin/nix-command-not-found $argv";
          onEvent = "fish_command_not_found";
        };
      };
    };

    programs.bash.enable = true;
    programs.nushell.enable = true;
    programs.zsh.enable = true;

    home.file = {
      ".local/bin/nix-command-not-found" = {
        executable = true;
        text = ''
          #!/usr/bin/env bash
          source ${pkgs.nix-index}/etc/profile.d/command-not-found.sh
          command_not_found_handle "$@"
        '';
      };
    };
  };
}
