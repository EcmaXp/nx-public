{ pkgs, lib, ... }:
pkgs.rustPlatform.buildRustPackage {
  pname = "nx-bash-safe";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  # The policy is data, not code, so it ships beside the binary rather than
  # being compiled in. The store path is baked so the binary can find it
  # without searching, and the store is immutable, which is what makes this
  # tier the one an overlay may only add to.
  NX_BASH_SAFE_POLICY_DIR = placeholder "out" + "/share/nx-bash-safe/policy";

  postInstall = ''
    mkdir -p $out/share/nx-bash-safe
    cp -r ${./policy} $out/share/nx-bash-safe/policy
  '';

  meta = with lib; {
    description = "Claude Code hook that auto-allows read-only Bash commands";
    platforms = platforms.unix;
    mainProgram = "nx-bash-safe";
  };
}
