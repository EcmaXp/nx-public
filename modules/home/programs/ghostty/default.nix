{
  config,
  lib,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.programs.ghostty {
  home.symlink = {
    ".config/ghostty/config" = ./config;
  };

  home.file = lib.mkIf pkgs.stdenv.isDarwin {
    ".local/bin/ghostty" = {
      source = config.lib.file.mkOutOfStoreSymlink "/Applications/Ghostty.app/Contents/MacOS/ghostty";
    };
  };
}
