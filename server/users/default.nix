{ config, hostname, ... }:
let
  inherit (config.home) username;
in
{
  programs.git = {
    userName = "${username}@${hostname}.local";
    userEmail = "${username}@${hostname}.local";
  };
}
