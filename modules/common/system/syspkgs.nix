{ pkgs, ... }:
{
  environment.systemPackages = import ./basepkgs.nix pkgs;
}
