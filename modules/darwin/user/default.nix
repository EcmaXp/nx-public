{ config, lib, ... }:
{
  config = lib.mkIf (config.nx.users.user.enable or false) {
    nx.darwin.users.user.enable = lib.mkDefault true;
  };
}
