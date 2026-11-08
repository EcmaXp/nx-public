{
  config,
  lib,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.standalone (
  lib.mkMerge [
    (import ../../common/system/nix-common.nix { inherit pkgs; })
    {
      targets.genericLinux.enable = pkgs.stdenv.hostPlatform.isLinux;

      nix.package = lib.mkForce pkgs.nix;

      home.packages = import ../../common/system/basepkgs.nix pkgs;
    }
  ]
)
