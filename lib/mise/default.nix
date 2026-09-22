{ lib, ... }:
{
  mise = {
    readConfigFiles = configFiles: import ./lock.nix { inherit lib configFiles; };
  };
}
