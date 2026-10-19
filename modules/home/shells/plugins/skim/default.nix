{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.skim {
  programs.skim = {
    enable = true;
    enableFishIntegration = false;
  };
}
