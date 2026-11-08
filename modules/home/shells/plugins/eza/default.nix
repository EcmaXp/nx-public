{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.eza {
  programs.eza = {
    enable = true;
    enableBashIntegration = false;
    enableFishIntegration = false;
    enableZshIntegration = false;
  };

  programs.bash = {
    shellAliases = {
      "eza" = "eza --git";
    };
  };

  programs.zsh = {
    shellAliases = {
      "eza" = "eza --git";
    };
  };

  programs.fish = {
    lazyAliases = {
      "eza" = "eza --git";
      "la" = "eza -a";
      "ll" = "eza -l";
      "lla" = "eza -la";
      "ls" = "eza";
      "lt" = "eza --tree";
    };
  };
}
