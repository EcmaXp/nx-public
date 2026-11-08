{ config, host, ... }:
let
  inherit (config.home) homeDirectory;
in
{
  home.sessionVariables = {
    NH_FLAKE = "${homeDirectory}/nx";
    NX_HOST = host;
  };

  home.file = {
    ".nx/host" = {
      text = host;
    };
  };

  home.scripts = {
    core = {
      bin = ../../../../tools/nx/bin/core;
      fish = ../../../../tools/nx/fish;
    };
  };

  programs.fish = {
    shellAbbrs = {
      # nx commands
      "nxbr" = "nx bin refresh";
      "nxc" = "nx clean";
      "nxp" = "nx repl";
      # nx os
      "nxob" = "nx os build";
      "nxos" = "nx os switch";
      "nxot" = "nx os test";
      # nx home
      "nxhb" = "nx home build";
      "nxhs" = "nx home switch";
    };
  };
}
