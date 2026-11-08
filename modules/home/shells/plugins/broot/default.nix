{
  config,
  lib,
  pkgs,
  ...
}:
let
  brFunction = pkgs.runCommand "br.fish" { nativeBuildInputs = [ config.programs.broot.package ]; } ''
    command broot --print-shell-function fish > $out
    grep -q '^function br ' $out
    ! grep -q '${builtins.storeDir}' $out
  '';
in
lib.nx.gate config.nx.home.shells.plugins.broot {
  programs.broot = {
    enable = true;
    enableFishIntegration = false;
  };

  xdg.configFile = {
    "fish/functions/br.fish" = {
      source = brFunction;
    };
  };
}
