{ pkgs, lib, ... }:
pkgs.rustPlatform.buildRustPackage {
  pname = "herdr-agent-state";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  # tests/claude.rs diffs our wire bytes against herdr's shipped python hook
  nativeCheckInputs = [ pkgs.python3 ];

  meta = with lib; {
    description = "Claude Code hook body that reports session identity to herdr";
    platforms = platforms.unix;
    mainProgram = "herdr-agent-state";
  };
}
