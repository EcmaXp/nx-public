{ inputs, pkgs, ... }:
(inputs.herdr.packages.${pkgs.stdenv.hostPlatform.system}.default).overrideAttrs (old: {
  patches = (old.patches or [ ]) ++ [
    ./event-stream.patch
    ./global-pane.patch
    ./focus-events.patch
    ./pixel-mouse.patch
  ];
})
