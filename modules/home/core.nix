{ lib, ... }:
{
  home.stateVersion = "24.11";

  programs.home-manager.enable = true;

  nix = {
    package = null;
    extraOptions = lib.mkForce "";
  };
}
