{
  config,
  lib,
  pkgs,
  ...
}:
let
  fzfInit =
    pkgs.runCommand "fzf-init.fish" { nativeBuildInputs = [ config.programs.fzf.package ]; }
      ''
        command fzf --fish > $out
        test -s $out
        ! grep -q '${builtins.storeDir}' $out
      '';
in
lib.nx.gate config.nx.home.shells.plugins.fzf {
  programs.fzf = {
    enable = true;
    enableFishIntegration = false;
  };

  programs.fish = {
    interactiveShellInit = lib.mkOrder 200 ''
      source ${fzfInit}
    '';
  };
}
