{ channels, ... }:
final: prev: {
  herdr = channels.nixpkgs-unstable.herdr;
}
