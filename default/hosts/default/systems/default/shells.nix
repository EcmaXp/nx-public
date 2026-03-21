{ pkgs, username, ... }:
{
  environment.shells = with pkgs; [
    bash
    zsh
    fish
  ];

  programs.fish = {
    enable = true;
  };

  users.users.${username} = {
    shell = pkgs.fish;
  };
}
