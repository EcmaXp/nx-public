{ channels, ... }:
final: prev: {
  uv = channels.nixpkgs-unstable.uv;
}
