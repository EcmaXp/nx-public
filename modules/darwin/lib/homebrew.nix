{
  config,
  lib,
  ...
}:
let
  userAppdir = "~/Applications";
  brewfilePath = "/etc/nix/Brewfile";
  toUserCask = name: {
    inherit name;
    args = {
      appdir = userAppdir;
    };
  };
in
{
  options.homebrew.userCasks = lib.mkOption {
    type = lib.types.listOf lib.types.str;
    default = [ ];
    description = ''
      Homebrew casks to install at `${userAppdir}` instead of the default `/Applications`.
      Each entry is translated to `{ name; args.appdir = "${userAppdir}"; }` and merged
      into `homebrew.casks`.
    '';
  };

  config = lib.mkIf config.nx.darwin.lib.homebrew.enable {
    homebrew = {
      casks = map toUserCask config.homebrew.userCasks;
    };

    environment = {
      variables = {
        HOMEBREW_BUNDLE_FILE = brewfilePath;
      };
      etc = {
        "nix/Brewfile" = {
          text = config.homebrew.brewfile;
        };
      };
    };
  };
}
