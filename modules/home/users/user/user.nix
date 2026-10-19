{ config, lib, ... }:
lib.nx.gate (config.nx.home.users.user or { }) {
  programs.git = {
    settings = {
      user = {
        name = "user";
        email = "user@example.com";
      };
    };
    signing = {
      # "op://Personal/User Git/public key"
      key = "ssh-ed25519 ...";
    };
  };
}
