{ config, lib, ... }:
lib.nx.gate config.nx.home.scripts.desktop {
  home.scripts = {
    "desktop-nx" = {
      bin = ../../../../tools/nx/bin/desktop;
    };
  };

  programs.fish = {
    shellAbbrs = {
      "nxlf" = "nx lock flake";
      "nxlm" = "nx lock mise";
      "nxlu" = "nx lock uv";
      "nxmi" = "nx mise install";
      "nxr" = "nx refresh";
      "nxs" = "nx sync";
      "nxsf" = "nx sync --forever";
      "nxsk" = "nx sync karabiner";
      "nxskf" = "nx sync karabiner --forever";
    };
  };
}
