{ lib, ... }:
let
  enable = description: lib.mkEnableOption description;
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
      desktop.enable = enable "Darwin desktop profile";
      server.enable = enable "server profile";
    };

    users = lib.mkOption {
      type = userOptions;
      default = { };
    };

    darwin = {
      lib.homebrew.enable = enable "Darwin Homebrew helper module";
      packages.desktop.enable = enable "Darwin desktop packages";
      programs.gnupg.enable = enable "Darwin GnuPG program defaults";
      system = {
        atrun.enable = enable "Darwin atrun defaults";
        desktop.enable = enable "Darwin desktop defaults";
        limits.enable = enable "Darwin limit defaults";
        macOS.enable = enable "macOS defaults";
        ramfs.enable = enable "Darwin RAM disk";
        security.enable = enable "Darwin security defaults";
      };
      users = lib.mkOption {
        type = userOptions;
        default = { };
      };
    };
  };
}
