{
  programs.git = {
    enable = true;
    lfs.enable = true;
    settings = {
      init.defaultbranch = "main";
      core.excludesfile = "~/.gitignore";
    };
  };

  home.file = {
    ".gitignore" = {
      source = ./git.gitignore;
    };
  };
}
