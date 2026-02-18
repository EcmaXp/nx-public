{ username, inputs, ... }:
{
  imports = [
    ./${username}/${username}.nix
    ./default
    ./secrets.nix
    inputs.nur.modules.homeManager.default
    inputs.nix-index-database.homeModules.nix-index
  ];
}
