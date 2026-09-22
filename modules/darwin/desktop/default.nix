{ config, lib, ... }:
{
  imports = [
    ../system/atrun.nix
    ../system/desktop.nix
    ../system/limits.nix
    ../system/macOS.nix
    ../system/ramfs.nix
    ../system/security.nix
  ];

  config = lib.mkIf config.nx.roles.desktop.enable {
    nx.darwin = {
      lib.homebrew.enable = lib.mkDefault true;
      packages.desktop.enable = lib.mkDefault true;
      programs.gnupg.enable = lib.mkDefault true;
      system = {
        atrun.enable = lib.mkDefault true;
        desktop.enable = lib.mkDefault true;
        limits.enable = lib.mkDefault true;
        macOS.enable = lib.mkDefault true;
        ramfs.enable = lib.mkDefault true;
        security.enable = lib.mkDefault true;
      };
    };
  };
}
