{
  config,
  lib,
  ...
}:
let
  inherit (config.lib.nix) mkRelativePath;

  fishCmd = {
    bin = "fish_add_path -gm";
    fish = "set --append fish_function_path";
  };

  getScriptPaths =
    name: cfg: type:
    let
      dir = cfg.${type} or null;
    in
    lib.optionals (dir != null) [
      (if cfg.fallback then "$HOME/.nx-fallback/${name}/${type}" else null)
      (mkRelativePath dir)
    ];

  makeAddPathCommand =
    type: path:
    lib.optionalString (path != null) ''
      ${fishCmd.${type}} ${path}
      for subdir in ${path}/*
        if test -d $subdir
          ${fishCmd.${type}} $subdir
        end
      end
    '';
in
{
  options.home.scripts = lib.mkOption {
    type = lib.types.attrsOf (
      lib.types.submodule {
        options = {
          bin = lib.mkOption {
            type = lib.types.nullOr lib.types.path;
            default = null;
          };
          fish = lib.mkOption {
            type = lib.types.nullOr lib.types.path;
            default = null;
          };
          fallback = lib.mkOption {
            type = lib.types.bool;
            default = true;
          };
        };
      }
    );
    default = { };
  };

  config = {
    home.file = lib.pipe config.home.scripts [
      (lib.mapAttrsToList (
        name: cfg:
        map (
          type:
          lib.mkIf (cfg.fallback && cfg.${type} != null) {
            ".nx-fallback/${name}/${type}" = {
              source = cfg.${type};
              recursive = true;
            };
          }
        ) (lib.attrNames fishCmd)
      ))
      lib.flatten
      lib.mkMerge
    ];

    programs.fish.shellInit = lib.pipe config.home.scripts [
      (lib.mapAttrsToList (
        name: cfg:
        lib.concatMap (type: map (makeAddPathCommand type) (getScriptPaths name cfg type)) (
          lib.attrNames fishCmd
        )
      ))
      lib.flatten
      (lib.concatStringsSep "\n")
    ];
  };
}
