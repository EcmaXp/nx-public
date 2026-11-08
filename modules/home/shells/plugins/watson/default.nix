{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.watson {
  programs.watson = {
    enable = true;
    enableFishIntegration = false;
  };
}
