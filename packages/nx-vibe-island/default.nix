{ pkgs, lib, ... }:
pkgs.rustPlatform.buildRustPackage {
  pname = "nx-vibe-island";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with lib; {
    description = "Claude Code hook that forwards events to the Vibe Island bridge";
    platforms = platforms.darwin;
    mainProgram = "nx-vibe-island";
  };
}
