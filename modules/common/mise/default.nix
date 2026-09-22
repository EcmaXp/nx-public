{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.mise;
  # nix-darwin gives generated /etc sources mkDefault priority.
  mkDefaultSource = lib.mkOverride 900;
  inherit (import ../../../lib/mise { inherit lib; }) mise;
  inherit (mise.readConfigFiles cfg.configFiles) configNames lockFiles mergedLock;
in
{
  options.programs.mise = {
    enable = lib.mkEnableOption "mise";
    package = lib.mkPackageOption pkgs "mise" { nullable = true; };
    configFiles = lib.mkOption {
      type = lib.types.listOf lib.types.path;
      default = [ ];
      description = "TOML files linked into /etc/mise/conf.d, with sibling .lock files for tools.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = builtins.length (lib.unique configNames) == builtins.length configNames;
        message = "programs.mise.configFiles must have unique file names.";
      }
    ];

    programs.mise.configFiles = lib.mkBefore [ ./settings.toml ];

    environment = {
      systemPackages = lib.optional (cfg.package != null) cfg.package;
      etc =
        lib.listToAttrs (
          map (file: {
            name = "mise/conf.d/${builtins.unsafeDiscardStringContext (builtins.baseNameOf file)}";
            value = {
              source = mkDefaultSource file;
            };
          }) cfg.configFiles
        )
        // {
          "mise/miserc.toml" = {
            source = mkDefaultSource ./miserc.toml;
          };
          "mise/mise.lock" = lib.mkIf (lockFiles != [ ]) {
            source = (pkgs.formats.toml { }).generate "mise.lock" mergedLock;
          };
        };
    };
  };
}
