{ config, lib, ... }:
lib.nx.gate config.nx.home.shells.plugins.starship {
  home.symlink = {
    ".config/starship.toml" = ./starship.toml;
    ".config/fish/async-starship.fish" = ./async-starship.fish;
  };

  programs.fish = {
    interactiveShellInit = lib.mkOrder 100 ''
      source $HOME/.config/fish/async-starship.fish
    '';
  };

  programs.starship = {
    enable = true;
    enableFishIntegration = false;
  };
}
