{
  config,
  lib,
  pkgs,
  inputs,
  ...
}:
let
  mkRelativePath = lib.nx.mkRelativePath inputs.self config.home.homeDirectory;
  inherit (inputs.home-manager.lib) hm;
  nixPath = "${config.home.homeDirectory}/nx";

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

  makeFallbackFiles =
    type: files:
    lib.listToAttrs (
      map (file: {
        name = ".nx/fallback/${type}/${file.name}";
        value.source = file.path;
      }) files
    );

  scriptDirs =
    type:
    lib.reverseList (
      lib.concatLists (
        lib.mapAttrsToList (
          _: cfg: lib.optional (cfg.${type} != null) (mkRelativePath cfg.${type})
        ) config.home.scripts
      )
    );

  completionDirs = lib.reverseList (
    lib.concatLists (
      lib.mapAttrsToList (
        _: cfg:
        lib.optional (cfg.fish != null && builtins.pathExists (cfg.fish + "/completions")) (
          (mkRelativePath cfg.fish) + "/completions"
        )
      ) config.home.scripts
    )
  );

  manifestText = paths: lib.concatStringsSep "\n" paths + "\n";
  nxBinPath = "${nixPath}/tools/nx/bin/core/nx-bin";
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
      [
        {
          ".nx/manifest/bin" = {
            text = manifestText (scriptDirs "bin");
          };
          ".nx/manifest/fish" = {
            text = manifestText (scriptDirs "fish");
          };
          ".nx/manifest/fish-completions" = {
            text = manifestText completionDirs;
          };
        }
      ]
      ++ lib.concatLists (
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

    programs.fish.shellInit = ''
      fish_add_path -gm $HOME/.nx/fallback/bin
      fish_add_path -gm $HOME/.nx/bin
      set --append fish_function_path $HOME/.nx/fish $HOME/.nx/fallback/fish
      set --append fish_complete_path $HOME/.nx/fish/completions $HOME/.nx/fallback/fish/completions
    '';

    home.activation.refreshNxBin = hm.dag.entryAfter [ "writeBoundary" ] ''
      $DRY_RUN_CMD ${pkgs.fish}/bin/fish "${nxBinPath}" refresh
    '';
  };
}
