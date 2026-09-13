{ pkgs, lib, ... }:
pkgs.rustPlatform.buildRustPackage {
  pname = "nx-comment-scan";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with lib; {
    description = "Claude Code hook that counts the comment lines an edit added";
    platforms = platforms.unix;
    mainProgram = "nx-comment-scan";
  };
}
