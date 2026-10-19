{ channels, ... }:
final: prev: {
  gomi = channels.nixpkgs-unstable.gomi.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [
      ./parallel-prune.patch
    ];
  });
}
