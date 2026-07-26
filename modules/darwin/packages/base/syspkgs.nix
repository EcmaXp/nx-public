{ pkgs, ... }:
{
  environment.systemPackages = (
    # NixOS pkgs for nix-darwin
    # https://github.com/NixOS/nixpkgs/blob/23.11/nixos/modules/config/system-path.nix
    with pkgs;
    [
      bashInteractive
      bzip2
      coreutils-full
      cpio
      curl
      diffutils
      findutils
      gawk
      getconf
      getent
      gnugrep
      gnupatch
      gnused
      gnutar
      gzip
      less
      ncurses
      netcat
      openssh
      procps
      stdenv.cc.libc
      time
      util-linux
      which
      xz
      zstd
    ]
  );
}
