{
  config,
  lib,
  pkgs,
  ...
}:
let
  kubeswitchInit = pkgs.runCommand "kubeswitch-init.fish" { } ''
    ${pkgs.kubeswitch}/bin/switcher init fish > $out
  '';
in
lib.nx.gate config.nx.home.shells.plugins.kubeswitch {
  programs.fish = {
    interactiveShellInit = ''
      source ${kubeswitchInit}
    '';
    shellAbbrs = {
      "kc" = "kubectx";
      "kn" = "kubens";
      "gkc" = "command kubectx";
    };
    functions = {
      "kubectx" = {
        body = "kubeswitch set-context $argv";
      };
      "kubens" = {
        body = "kubeswitch namespace $argv";
      };
    };
  };
}
