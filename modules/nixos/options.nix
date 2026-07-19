{ lib, ... }:
let
  userOptions = lib.types.attrsOf (
    lib.types.submodule {
      options.enable = lib.mkEnableOption "this user profile";
    }
  );
in
{
  options.nx = {
    primaryUser = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
    };

    roles = {
      desktop.enable = lib.mkEnableOption "desktop profile";
      server.enable = lib.mkEnableOption "NixOS server profile";
    };

    users = lib.mkOption {
      type = userOptions;
      default = { };
    };

    nixos.system.server.enable = lib.mkEnableOption "NixOS server system module";
  };
}
