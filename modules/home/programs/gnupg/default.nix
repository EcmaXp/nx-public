{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.gnupg {
  home.sessionVariables = {
    PINENTRY_PROGRAM = "pinentry-mac";
  };

  home.symlink = {
    ".gnupg/gpg-agent.conf" = ./gpg-agent.conf;
  };
}
