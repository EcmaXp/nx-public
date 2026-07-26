{
  config,
  inputs,
  lib,
  ...
}:
let
  inherit (inputs.home-manager.lib) hm;
  pluginCacheDir = "${config.home.homeDirectory}/.terraform.d/plugin-cache";
in
lib.nx.gate config.nx.home.programs.terraform {
  home.file = {
    ".terraformrc" = {
      text = ''
        plugin_cache_dir = "${pluginCacheDir}"
      '';
    };
  };

  home.activation = {
    terraformPluginCacheDir = hm.dag.entryAfter [ "writeBoundary" ] ''
      $DRY_RUN_CMD mkdir -p "${pluginCacheDir}"
    '';
  };
}
