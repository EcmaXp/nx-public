{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.man {
  programs.man = {
    enable = true;
    generateCaches = false;
  };
}
