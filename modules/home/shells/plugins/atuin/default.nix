{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.atuin {
  programs.atuin = {
    enable = true;
    flags = [ "--disable-up-arrow" ];
    settings = {
      style = "compact";
      inline_height = 40;
      enter_accept = false;
    };
  };
}
