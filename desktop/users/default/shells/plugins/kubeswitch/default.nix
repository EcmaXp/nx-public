{ pkgs, ... }:
{
  home.file = {
    ".config/fish/functions/kubeswitch.fish" = {
      executable = true;
      text =
        builtins.readFile
          (pkgs.runCommand "kubeswitch-init-fish" { } ''
            ${pkgs.kubeswitch}/bin/switcher init fish > $out
            substituteInPlace $out \
              --replace '\rm -f "$KUBECONFIG"' 'rm -f "$KUBECONFIG"'
          '').outPath;
    };
  };

  programs.fish = {
    shellAbbrs = {
      "kc" = "kubectx";
      "kn" = "kubens";
      "gkc" = "command kubectx";
      "gkn" = "command kubens";
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
