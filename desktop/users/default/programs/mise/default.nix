{ config, pkgs, ... }:
{
  programs.mise = {
    enable = true;
    package = config.lib.brew.wrapPackage {
      name = "mise";
      pkg = pkgs.mise;
    };
    globalConfig = {
      settings = {
        legacy_version_file_disable_tools = [
          "terraform"
        ];
        idiomatic_version_file_enable_tools = [
          "node"
          "python"
        ];
      };
    };
  };
}
