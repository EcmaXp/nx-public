{
  hostname,
  username,
  lib,
  ...
}:
let
  flake = "/etc/nix-darwin";
  home = "/Users/${username}";
  sudo = "/usr/bin/sudo";
in
{
  networking.hostName = hostname;
  system.primaryUser = username;

  time.timeZone = lib.mkDefault "GMT";

  system.activationScripts = {
    postActivation = {
      text = ''
        stat "${flake}" >/dev/null || \
          ${sudo} ln -s "${home}/nix" "${flake}"
      '';
    };
  };

  users.users = {
    ${username} = {
      inherit home;
    };
  };
}
