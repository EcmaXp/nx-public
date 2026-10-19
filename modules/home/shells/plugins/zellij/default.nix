{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.zellij {
  programs.zellij = {
    enable = true;
  };
}
