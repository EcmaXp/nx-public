{
  config,
  lib,
  pkgs,
  ...
}:
let
  fishHook =
    pkgs.runCommand "direnv-hook.fish" { nativeBuildInputs = [ config.programs.direnv.package ]; }
      ''
        cat ${./should-export.fish} > $out
        command direnv hook fish \
          | sed "s|\"$(command -v direnv)\"|command direnv|g" \
          | sed 's/command direnv export fish/__direnv_should_export; and &/' >> $out
        test "$(grep -c '__direnv_should_export; and command direnv export fish' $out)" = 3
        ! grep -q '${builtins.storeDir}' $out
      '';
in
lib.nx.gate config.nx.home.shells.plugins.direnv {
  programs = {
    direnv = {
      enable = true;
      nix-direnv.enable = true;
      enableFishIntegration = false;
    };
  };

  home.symlink = {
    ".config/direnv/direnvrc" = ./direnvrc;
  };

  xdg.configFile = {
    "fish/conf.d/direnv.fish" = {
      text = ''
        status is-interactive; and source ${fishHook}
      '';
    };
  };
}
