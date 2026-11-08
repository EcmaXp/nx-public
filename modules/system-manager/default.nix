{
  inputs,
  pkgs,
  ...
}:
{
  nix = {
    enable = true;
    settings = {
      experimental-features = "nix-command flakes auto-allocate-uids cgroups";
      auto-allocate-uids = true;
      extra-system-features = "uid-range";
      build-users-group = "nixbld";
      trusted-users = [ "@sudo" ];
    };
  };

  services.userborn.enable = false;

  environment.systemPackages = [
    inputs.self.packages.${pkgs.stdenv.hostPlatform.system}.system-manager
  ];

  systemd.services = {
    nix-daemon-reload = {
      description = "Restart nix-daemon after nix.conf changes";
      wantedBy = [ "system-manager.target" ];
      serviceConfig = {
        Type = "oneshot";
      };
      script = ''
        hash=$(${pkgs.coreutils}/bin/sha256sum < /etc/nix/nix.conf)
        stamp=/run/nix-daemon-reload.sha256
        if [ "$hash" != "$(${pkgs.coreutils}/bin/cat $stamp 2> /dev/null)" ]; then
          ${pkgs.systemd}/bin/systemctl try-restart nix-daemon.service
          echo "$hash" > $stamp
        fi
      '';
    };
  };
}
