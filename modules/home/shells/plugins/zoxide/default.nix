{
  config,
  lib,
  pkgs,
  ...
}:
let
  zoxideInit =
    pkgs.runCommand "zoxide-init.fish" { nativeBuildInputs = [ config.programs.zoxide.package ]; }
      ''
        command zoxide init fish > $out
        test -s $out
        ! grep -q '${builtins.storeDir}' $out
      '';
in
lib.nx.gate config.nx.home.shells.plugins.zoxide {
  programs.zoxide = {
    enable = true;
    enableFishIntegration = false;
  };

  programs.fish = {
    interactiveShellInit = ''
      source ${zoxideInit}
    '';
  };
}
