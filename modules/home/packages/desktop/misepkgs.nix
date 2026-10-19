{ config, lib, ... }:
lib.nx.gate config.nx.home.packages.desktop {
  programs.mise = {
    enable = lib.mkDefault true;
    configFiles = [ ./tools.toml ];
  };
}
