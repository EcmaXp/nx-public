{ config, ... }:
let
  inherit (config.home) homeDirectory;
in
{
  home.sessionVariables = {
    NH_FLAKE = "${homeDirectory}/nx";
  };

  home.scripts = {
    "default-nx" = {
      bin = ./bin;
      fish = ./fish;
    };
  };

  programs.fish = {
    shellAbbrs = {
      # nx commands
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
