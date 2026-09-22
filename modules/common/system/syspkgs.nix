{ pkgs, ... }:
{
  environment.systemPackages = (
    with pkgs;
    [
      file
      git
      hostname
      inetutils
      micro
      moreutils
      nano
      nh
      parallel
      screen
      tmux
      tree
      vim
      watch
      wget
    ]
  );
}
