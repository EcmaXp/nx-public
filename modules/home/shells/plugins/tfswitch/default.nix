{
  config,
  lib,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.shells.plugins.tfswitch {
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
