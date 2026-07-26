{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.aerospace {
  home.symlink = {
    ".aerospace.toml" = ./aerospace.toml;
  };
}
