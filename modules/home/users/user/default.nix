{ config, lib, ... }:
{
  imports = [
    ./user.nix
  ];

  config = lib.mkIf config.nx.users.user.enable {
    nx.home.users.user.enable = lib.mkDefault true;
  };
}
