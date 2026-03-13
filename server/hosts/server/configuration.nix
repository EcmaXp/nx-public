# /etc/nixos/configuration.nix
# nixos-help command or https://search.nixos.org/options

{
  ...
}:
{
  imports = [
    ./hardware-configuration.nix
  ];
}
