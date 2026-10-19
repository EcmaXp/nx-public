{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.tmux {
  programs.tmux = {
    enable = true;
  };
}
