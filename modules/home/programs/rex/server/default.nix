{ config, lib, ... }:
let
  unconditionalLingerReminder = "services.rex: the Rex server runs as a systemd user service";
in
{
  options.warnings = lib.mkOption {
    apply =
      if config.nx.home.programs.rex.server.enable or false then
        builtins.filter (w: !lib.hasPrefix unconditionalLingerReminder w)
      else
        lib.id;
  };

  config = lib.mkIf (config.nx.home.programs.rex.server.enable or false) {
    services.rex = {
      enable = true;
      tailscale = {
        enable = true;
        hostname = "server-rex";
      };
    };

    programs.ssh = {
      enable = true;
      enableDefaultConfig = false;
    };
  };
}
