{
  hostname,
  username,
  lib,
  pkgs,
  ...
}:
let
  flake = "/etc/nixos";
  home = "/home/${username}";
  sudo = "${pkgs.sudo}/bin/sudo";
in
{
  networking.hostName = hostname;

  time.timeZone = lib.mkDefault "Etc/UTC";

  system.activationScripts = {
    postActivation = {
      text = ''
        stat "${flake}" >/dev/null || \
          ${sudo} ln -s "${home}/nix" "${flake}"

        ${sudo} chown ${username}:users -R "${home}/nix"
      '';
    };
  };

  users.users = {
    ${username} = {
      inherit home;
    };
  };
}
