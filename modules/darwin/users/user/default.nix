{ config, lib, ... }:
lib.nx.gate (config.nx.darwin.users.user or { }) {
  system.defaults = {
    loginwindow = {
      LoginwindowText = lib.strings.concatLines [
        config.networking.hostName
        "user@example.com"
      ];
    };
  };
}
