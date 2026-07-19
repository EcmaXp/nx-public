{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.granted {
  programs.fish = {
    shellAliases = {
      "assume" = "source /opt/homebrew/bin/assume.fish";
    };
  };
}
