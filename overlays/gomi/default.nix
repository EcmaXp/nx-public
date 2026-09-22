{ channels, ... }:
final: prev: {
  gomi = channels.nixpkgs-unstable.gomi;
}
