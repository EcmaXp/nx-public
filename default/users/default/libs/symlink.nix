{
  config,
  lib,
  inputs,
  ...
}:
let
  inherit (config.lib.nix) mkRelativePath;
  inherit (inputs.home-manager.lib) hm;
  nixPath = "${config.home.homeDirectory}/nx";
in
{
  options.home.symlink = lib.mkOption {
    type = lib.types.attrsOf lib.types.path;
    default = { };
  };

  options.home.repoSymlink = lib.mkOption {
    type = lib.types.attrsOf lib.types.str;
    default = { };
  };

  config = {
    home.file = lib.mapAttrs (name: source: {
      source = config.lib.file.mkOutOfStoreSymlink (mkRelativePath source);
      force = true;
    }) config.home.symlink;

    home.activation.repoSymlinks = lib.mkIf (config.home.repoSymlink != { }) (
      hm.dag.entryAfter [ "writeBoundary" ] (
        lib.concatStringsSep "\n" (
          lib.mapAttrsToList (name: target: ''
            $DRY_RUN_CMD ln -sfn "${target}" "${nixPath}/${name}"
          '') config.home.repoSymlink
        )
      )
    );
  };
}
