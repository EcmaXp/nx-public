{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.ghostty {
  home.symlink = {
    ".config/ghostty/config" = ./config;
  };

  programs.fish.shellInit = ''
    fish_add_path -gm /Applications/Ghostty.app/Contents/MacOS
  '';
}
