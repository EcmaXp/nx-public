{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (pkgs.stdenv.hostPlatform) system;
  # NUR's charmbracelet packages default `system` to `pkgs.system`, a
  # deprecated alias that emits an evaluation warning.
  pinSystem = lib.mapAttrs (
    _: pkg:
    if (pkg.override.__functionArgs or { }) ? system then pkg.override { inherit system; } else pkg
  );
  allowBroken =
    pkg:
    pkg.overrideAttrs (old: {
      meta = (old.meta or { }) // {
        broken = false;
      };
    });
in
lib.nx.gate config.nx.home.packages.desktop {
  home.packages = with pinSystem pkgs.nur.repos.charmbracelet; [
    (allowBroken charm)
    crush
    freeze
    glow
    gum
    mods
    pop
    skate
    soft-serve
    vhs
  ];
}
