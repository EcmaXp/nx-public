{
  config,
  lib,
  pkgs,
  ...
}:
let
  kubeswitchInit =
    pkgs.runCommand "kubeswitch-init.fish" { nativeBuildInputs = [ pkgs.kubeswitch ]; }
      ''
        command switcher init fish > init.fish
        grep -q 'complete --do-complete "switcher "' init.fish
        sed '/^if type -q "switcher"$/,/^end$/d' init.fish > $out
        ! grep -q 'complete --do-complete' $out
        ! grep -q '${builtins.storeDir}' $out
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
