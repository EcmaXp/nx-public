{ config, lib, ... }:
let
  user = config.nx.primaryUser;
in
lib.nx.gate config.nx.darwin.programs.caffeinate {
  security.sudo.extraConfig = ''
    ${user} ALL = (root) NOPASSWD: /usr/bin/caffeinate -dimsu
    ${user} ALL = (root) NOPASSWD: /usr/bin/pkill -x -U root caffeinate
  '';
}
