{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.nx.darwin.system.spotlight;
  sync = pkgs.writeShellApplication {
    name = "nx-spotlight-sync";
    runtimeInputs = [
      pkgs.coreutils
      pkgs.jq
    ];
    text = ''
      wantJson=${lib.escapeShellArg (builtins.toJSON cfg.exclusions)}
      declaredIds=${
        lib.escapeShellArg (lib.concatMapStringsSep "\n" toString (lib.attrValues config.homebrew.masApps))
      }
      brewUser=${lib.escapeShellArg config.homebrew.user}
      masPath=${lib.escapeShellArg "${config.homebrew.prefix}/bin:${lib.makeBinPath [ pkgs.mas ]}"}
    ''
    + builtins.readFile ./spotlight.sh;
  };
in
lib.nx.gate cfg {
  system.activationScripts = {
    homebrew = {
      text = lib.mkMerge [
        (lib.mkBefore "if ${lib.getExe sync}; then :")
        (lib.mkAfter ''
          else
            printf >&2 '\e[1;33mwarning: brew bundle skipped, see above; run nx os switch again\e[0m\n'
          fi
        '')
      ];
    };
  };
}
