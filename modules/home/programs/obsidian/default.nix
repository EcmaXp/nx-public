{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.obsidian {
  programs.fish.shellInit = ''
    fish_add_path -gm /Applications/Obsidian.app/Contents/MacOS
  '';
}
