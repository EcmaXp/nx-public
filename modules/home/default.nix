{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.mise;
  mkRelativePath = lib.nx.mkRelativePath inputs.self config.home.homeDirectory;
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

    xdg.configFile =
      lib.mapAttrs
        (_: source: {
          source = config.lib.file.mkOutOfStoreSymlink (mkRelativePath source);
          force = true;
        })
        (
          lib.listToAttrs (
            map (file: {
              name = "mise/conf.d/${builtins.baseNameOf file}";
              value = file;
            }) cfg.configFiles
          )
          // {
            "mise/miserc.toml" = ../common/mise/miserc.toml;
          }
        );
  };
}
