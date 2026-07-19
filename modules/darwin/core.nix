{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.nx;
  flake = "/etc/nix-darwin";
  sudo = "/usr/bin/sudo";
in
lib.mkIf (cfg.primaryUser != null) {
  snowfallorg.users.${cfg.primaryUser} = {
    create = false;
    home.enable = false;
  };

  system.primaryUser = cfg.primaryUser;

  time.timeZone = lib.mkDefault "GMT";

  system.activationScripts.postActivation.text = ''
    stat "${flake}" >/dev/null || \
      ${sudo} ln -s "/Users/${cfg.primaryUser}/nx" "${flake}"
  '';

  users.users.${cfg.primaryUser} = {
    home = "/Users/${cfg.primaryUser}";
    shell = pkgs.fish;
  };
}
