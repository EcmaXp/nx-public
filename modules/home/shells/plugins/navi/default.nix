{
  config,
  lib,
  pkgs,
  ...
}:
let
  naviInit =
    pkgs.runCommand "navi-init.fish" { nativeBuildInputs = [ config.programs.navi.package ]; }
      ''
        command navi widget fish > $out
        test -s $out
        ! grep -q '${builtins.storeDir}' $out
      '';
in
lib.nx.gate config.nx.home.shells.plugins.navi {
  programs.navi = {
    enable = true;
    enableFishIntegration = false;
  };

  programs.fish = {
    interactiveShellInit = ''
      source ${naviInit}
    '';
  };
}
