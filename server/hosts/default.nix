{ hostname, ... }:
{
  imports = [
    ./default
    ./${hostname}/configuration.nix
  ];
}
