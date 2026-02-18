{ username, pkgs, ... }:
let
  sudo = "${pkgs.sudo}/bin/sudo";
in
{
  imports = [
    ./configuration.nix
  ];

  system.activationScripts = {
    postActivation = {
      text = ''
        stat "/home/${username}/nix" >/dev/null || \
          ${sudo} -u ${username} ln -sf "/Users/${username}/nix" "/home/${username}/nix"
      '';
    };
  };
}
