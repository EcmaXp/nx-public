{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.mise;
  inherit (import ../../../lib/mise { inherit lib; }) mise;
  inherit (mise.readConfigFiles cfg.configFiles) configNames lockFiles mergedLock;
in
{
  options.programs.mise = {
    configFiles = lib.mkOption {
      type = lib.types.listOf lib.types.path;
      default = [ ];
      description = "TOML files linked into mise/conf.d, with sibling .lock files for tools.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = builtins.length (lib.unique configNames) == builtins.length configNames;
        message = "programs.mise.configFiles must have unique file names.";
      }
    ];

    home.sessionPath = lib.mkBefore [ "${config.xdg.dataHome}/mise/shims" ];

    programs.mise = {
      enableBashIntegration = false;
      enableFishIntegration = false;
      enableNushellIntegration = false;
      enableZshIntegration = false;
      configFiles = lib.mkBefore [ ../../common/mise/settings.toml ];
    };

    xdg.configFile =
      lib.listToAttrs (
        map (file: {
          name = "mise/conf.d/${builtins.baseNameOf file}";
          value = {
            source = lib.mkDefault file;
          };
        }) cfg.configFiles
      )
      // {
        "mise/miserc.toml" = {
          source = lib.mkDefault ../../common/mise/miserc.toml;
        };
        "fish/conf.d/mise-activate.fish" = {
          text = "";
        };
        "mise/mise.lock" = lib.mkIf (lockFiles != [ ]) {
          source = (pkgs.formats.toml { }).generate "mise.lock" mergedLock;
        };
      };
  };
}
