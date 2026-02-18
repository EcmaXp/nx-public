{
  home.sessionVariables = {
    PINENTRY_PROGRAM = "pinentry-mac";
  };

  home.symlink = {
    ".gnupg/gpg-agent.conf" = ./gpg-agent.conf;
  };
}
