{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.yazi {
  programs.yazi = {
    enable = true;
    shellWrapperName = "yy";
  };
}
