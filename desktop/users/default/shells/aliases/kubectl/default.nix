{
  home.symlink = {
    ".config/fish/kubectl_aliases.fish" = ./kubectl_aliases.fish;
  };

  programs.fish = {
    interactiveShellInit = ''
      source $HOME/.config/fish/kubectl_aliases.fish
    '';
  };
}
