# Server role: default git identity derived from the user and host name.
{
  config,
  host,
  lib,
  ...
}:
let
  user = config.snowfallorg.user.name;
in
{
  config = lib.mkIf config.nx.roles.server.enable {
    nx.home.server.enable = lib.mkDefault true;

    programs.git.settings.user = {
      name = "${user}@${host}.local";
      email = "${user}@${host}.local";
    };
  };
}
