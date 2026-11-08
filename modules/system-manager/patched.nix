{
  inputs,
  pkgs,
  patches ? [ ./system-manager.patch ],
}:
rec {
  src = pkgs.applyPatches {
    name = "system-manager-patched";
    src = inputs.system-manager;
    inherit patches;
  };
  lib = import "${src}/nix/lib.nix" {
    inherit (inputs) nixpkgs;
    inherit (inputs.system-manager.inputs) userborn;
  };
}
