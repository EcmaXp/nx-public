{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.direnv {
  programs = {
    direnv = {
      enable = true;
      nix-direnv.enable = true;
    };
  };

  home.symlink = {
    ".config/direnv/direnvrc" = ./direnvrc;
  };
}
