{
  config,
  lib,
  pkgs,
  osConfig ? { },
  ...
}:
lib.nx.gate config.nx.home.programs.mise {
  programs.mise = {
    enable = true;
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
}
