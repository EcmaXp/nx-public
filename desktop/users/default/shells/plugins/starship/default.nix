{ lib, ... }:
{
  home.symlink = {
    ".config/starship.toml" = ./starship.toml;
    ".config/fish/async-starship.fish" = ./async-starship.fish;
  };

  programs.fish = {
    interactiveShellInit = lib.mkAfter ''
      source $HOME/.config/fish/async-starship.fish
    '';
  };

  programs.starship = {
    enable = true;
  };
}
