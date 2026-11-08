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
  imports = [
    ./default/default.nix
    ./desktop/default.nix
    ./user/default.nix
    ./user/default.nix
    ./server/default.nix
  ];

  config = lib.mkIf cfg.enable {
    programs.mise = {
      package = lib.mkDefault pkgs.nx.mise;
    };
  };
}
