{ inputs, ... }:
{
  imports = [
    ./darwin.nix
    ./homebrew.nix
    ./nix.nix
    ./packages
    inputs.home-manager.darwinModules.home-manager
    inputs.nix-index-database.darwinModules.nix-index
    inputs.nur.modules.darwin.default
    inputs.ragenix.darwinModules.default
  ];
}
