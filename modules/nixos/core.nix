{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.nx;
  flake = "/etc/nixos";
  sudo = "${pkgs.sudo}/bin/sudo";
in
lib.mkMerge [
  {
    system.stateVersion = lib.mkDefault "24.11";
    system.configurationRevision = inputs.self.rev or inputs.self.dirtyRev or null;

    time.timeZone = lib.mkDefault "Etc/UTC";

    environment.enableAllTerminfo = true;

    nix = {
      gc = {
        automatic = true;
        dates = "daily";
        options = "--delete-older-than 30d";
      };

      optimise = {
        automatic = true;
        dates = [ "18:00" ];
      };
    };
  }

  (lib.mkIf (cfg.primaryUser != null) {
    snowfallorg.users.${cfg.primaryUser} = {
      create = false;
      home.enable = false;
    };

    system.activationScripts.postActivation.text = ''
      stat "${flake}" >/dev/null || \
        ${sudo} ln -s "/home/${cfg.primaryUser}/nx" "${flake}"

      ${sudo} chown ${cfg.primaryUser}:users -R "/home/${cfg.primaryUser}/nx"
    '';

    users.groups.${cfg.primaryUser} = { };
    users.users.${cfg.primaryUser} = {
      home = "/home/${cfg.primaryUser}";
      shell = pkgs.fish;
      isNormalUser = lib.mkDefault true;
      group = lib.mkForce cfg.primaryUser;
    };
  })
]
