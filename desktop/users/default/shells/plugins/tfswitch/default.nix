{ pkgs, ... }:
{
  home.packages = (
    with pkgs;
    [
      tfswitch
    ]
  );

  home.symlink = {
    ".config/fish/tfswitch.fish" = ./tfswitch.fish;
  };

  programs.fish = {
    interactiveShellInit = ''
      source $HOME/.config/fish/tfswitch.fish
    '';
  };
}
