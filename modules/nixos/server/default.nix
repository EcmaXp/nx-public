{ config, lib, ... }:
{
  config = lib.mkIf config.nx.roles.server.enable {
    nx.nixos.system.server.enable = lib.mkDefault true;
  };
}
