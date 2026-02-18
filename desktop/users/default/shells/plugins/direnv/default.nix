{
  programs = {
    direnv = {
      enable = true;
      nix-direnv.enable = true;
    };
  };

  home.symlink = {
    ".config/direnv/direnvrc" = ./direnvrc;
  };
}
