{ lib, ... }:
{
  mise = {
    readConfigFiles = pkgs: configFiles: import ./lock.nix { inherit lib pkgs configFiles; };
  };
}
