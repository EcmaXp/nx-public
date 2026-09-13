{
  config,
  lib,
  pkgs,
  osConfig ? { },
  ...
}:
{
  config = lib.mkIf config.nx.home.programs.mise.enable {
    home.sessionPath = lib.mkBefore [ "${config.xdg.dataHome}/mise/shims" ];

    programs.mise = {
      enable = true;
      enableBashIntegration = false;
      enableFishIntegration = false;
      enableNushellIntegration = false;
      enableZshIntegration = false;
      package = pkgs.nx.mise;
      globalConfig = {
        settings = {
          lockfile = true;
          locked = true;
          minimum_release_age = "7d";
          legacy_version_file_disable_tools = [
            "terraform"
          ];
          idiomatic_version_file_enable_tools = [
            "node"
            "python"
          ];
          ruby = {
            compile = false;
          };
        };
      };
    };

    home.symlink = {
      ".config/mise/mise.lock" = ../../packages/desktop/misepkgs.lock.toml;
    };

    xdg.configFile = {
      "fish/conf.d/mise-activate.fish" = {
        text = "";
      };
    };
  };
}
