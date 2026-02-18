{ username, ... }:
{
  imports = [
    ./${username}/system.nix
  ];
}
