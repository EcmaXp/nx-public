{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.mise;
in
{
  imports = [ ../../common/mise ];

  config = lib.mkIf cfg.enable {
    environment.variables.PATH = lib.mkBefore [ "/usr/local/share/mise/shims" ];

    programs.mise = {
      package = lib.mkDefault pkgs.nx.mise;
    };
  };
}
