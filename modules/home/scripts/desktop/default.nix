{ config, lib, ... }:
{
  imports = [
    ./nx.nix
  ];

  config = lib.mkIf config.nx.home.scripts.desktop.enable {
    home.scripts = {
      desktop = {
        bin = ./bin;
      };
    };
  };
}
