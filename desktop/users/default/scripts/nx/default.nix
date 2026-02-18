{
  home.scripts = {
    "desktop-nx" = {
      bin = ./bin;
    };
  };

  programs.fish = {
    shellAbbrs = {
      "nxlf" = "nx lock flake";
      "nxlu" = "nx lock uv";
      "nxmu" = "nx mise upgrade";
      "nxr" = "nx refresh";
      "nxs" = "nx sync";
      "nxsf" = "nx sync --forever";
      "nxsk" = "nx sync karabiner";
      "nxskf" = "nx sync karabiner --forever";
    };
  };
}
