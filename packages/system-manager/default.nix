{
  inputs,
  lib,
  pkgs,
  ...
}:
let
  inherit (import ../../modules/system-manager/patched.nix { inherit inputs pkgs; }) src;
  patched = pkgs.callPackage "${src}/nix/packages/wrapper.nix" {
    system-manager-unwrapped = pkgs.callPackage "${src}/package.nix" { };
  };
in
pkgs.symlinkJoin {
  name = "system-manager";
  paths = [ patched ];
  meta = {
    mainProgram = "system-manager";
    platforms = lib.platforms.linux;
  };
}
