{
  config,
  lib,
  ...
}:
let
  inherit (config.lib.nix) mkRelativePath;

  scriptTypes = [
    "bin"
    "fish"
  ];

  fishCmd = {
    bin = "fish_add_path -gm";
    fish = "set --append fish_function_path";
    fishCompletions = "set --append fish_complete_path";
  };

  collectFiles =
    dir:
    let
      entries = builtins.readDir dir;
    in
    lib.concatMap (
      name:
      let
        path = dir + "/${name}";
      in
      if entries.${name} == "directory" then
        if name == "completions" then [ ] else collectFiles path
      else if name == ".gitkeep" then
        [ ]
      else
        [ { inherit name path; } ]
    ) (lib.attrNames entries);

  collectCompletions =
    dir:
    let
      completionsDir = dir + "/completions";
    in
    lib.optionals (builtins.pathExists completionsDir) (collectFiles completionsDir);

  makePathLoop =
    type: paths:
    let
      formattedPaths = lib.concatStringsSep " \\\n    " paths;
    in
    lib.optionalString (paths != [ ]) ''
      for dir in \
          ${formattedPaths}
        ${fishCmd.${type}} $dir
        for subdir in $dir/*
          if test -d $subdir
            if test (basename $subdir) = completions
              ${fishCmd.fishCompletions} $subdir
            else
              ${fishCmd.${type}} $subdir
            end
          end
        end
      end
    '';

  makeFallbackFiles =
    type: files:
    lib.listToAttrs (
      map (file: {
        name = ".nx-fallback/${type}/${file.name}";
        value.source = file.path;
      }) files
    );
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
    home.file = lib.mkMerge (
      lib.concatLists (
        lib.mapAttrsToList (
          _: cfg:
          lib.optionals cfg.fallback [
            (lib.mkIf (cfg.bin != null) (makeFallbackFiles "bin" (collectFiles cfg.bin)))
            (lib.mkIf (cfg.fish != null) (makeFallbackFiles "fish" (collectFiles cfg.fish)))
            (lib.mkIf (cfg.fish != null) (makeFallbackFiles "fish/completions" (collectCompletions cfg.fish)))
          ]
        ) config.home.scripts
      )
    );

    programs.fish.shellInit =
      let
        collectPaths =
          type:
          [ "$HOME/.nx-fallback/${type}" ]
          ++ lib.concatLists (
            lib.mapAttrsToList (
              _: cfg: lib.optional (cfg.${type} != null) (mkRelativePath cfg.${type})
            ) config.home.scripts
          );
      in
      lib.concatStringsSep "\n" (map (type: makePathLoop type (collectPaths type)) scriptTypes);
  };
}
