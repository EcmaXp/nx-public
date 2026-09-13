{ pkgs, lib, ... }:
pkgs.rustPlatform.buildRustPackage {
  pname = "nx-claude-statusline";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  meta = with lib; {
    description = "Claude Code status line: model, effort, directory, branch, worktree, context";
    platforms = platforms.unix;
    mainProgram = "nx-claude-statusline";
  };
}
