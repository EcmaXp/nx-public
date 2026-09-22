{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.onepassword {
  home.sessionVariables = {
    SSH_AUTH_SOCK = "${config.home.homeDirectory}/Library/Group Containers/2BUA8C4S2C.com.1password/t/agent.sock";
  };

  programs.git.signing = {
    signer = "/Applications/1Password.app/Contents/MacOS/op-ssh-sign";
    signByDefault = lib.mkDefault true;
    format = "ssh";
  };
}
