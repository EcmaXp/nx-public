# /etc/nixos/configuration.nix
# nixos-help command or https://search.nixos.org/options

{
  ...
}:
{
  # https://nixos.org/manual/nixos/stable/options#opt-system.stateVersion
  system.stateVersion = "23.11"; # Did you read the comment?

  imports = [
    ./hardware-configuration.nix
  ];

  # host
  networking.hostName = "server";

  nx = {
    primaryUser = "user";
    roles.server.enable = true;
  };
}
