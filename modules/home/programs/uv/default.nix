{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.uv {
  home.symlink = {
    ".config/uv/uv.toml" = ./uv.toml;
  };
}
