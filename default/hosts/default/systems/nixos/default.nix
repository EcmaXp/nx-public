{ inputs, ... }:
{
  imports = [
    ./nix.nix
    ./nixos.nix
    ./terminfo.nix
    inputs.home-manager.nixosModules.home-manager
    inputs.nix-index-database.nixosModules.nix-index
    inputs.nur.modules.nixos.default
    inputs.ragenix.nixosModules.default
  ];
}
