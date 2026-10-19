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
        ++ lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
          "${brewPrefix}/bin"
          "${brewPrefix}/sbin"
        ]
      ))
    ];

    home.sessionVariables = lib.mkIf pkgs.stdenv.hostPlatform.isDarwin {
      HOMEBREW_PREFIX = brewPrefix;
      HOMEBREW_CELLAR = "${brewPrefix}/Cellar";
      HOMEBREW_REPOSITORY = brewPrefix;
      INFOPATH = "${brewPrefix}/share/info:$INFOPATH";
    };

    home.sessionSearchVariables = lib.mkIf pkgs.stdenv.hostPlatform.isDarwin {
      MANPATH = [ "" ];
    };

    programs.bash.enable = true;
    programs.nushell.enable = true;
    programs.zsh.enable = true;
  };
}
