{ pkgs, ... }:
{
  environment.shells = with pkgs; [
    bash
    zsh
    fish
  ];

  programs.fish.enable = true;
}
