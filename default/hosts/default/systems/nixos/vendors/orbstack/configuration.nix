# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).

{ modulesPath, username, ... }:

{
  imports = [
    # Include the default lxd configuration.
    "${modulesPath}/virtualisation/lxc-container.nix"
    # Include the OrbStack-specific configuration.
    ./orbstack.nix
  ];

  users.users.${username} = {
    uid = 501;
    extraGroups = [ "wheel" ];

    # simulate isNormalUser, but with an arbitrary UID
    isSystemUser = true;
    group = "users";
    createHome = true;
    home = "/home/${username}";
    homeMode = "700";
    useDefaultShell = true;
  };

  security.sudo.wheelNeedsPassword = false;

  # This being `true` leads to a few nasty bugs, change at your own risk!
  users.mutableUsers = false;

  networking = {
    dhcpcd.enable = false;
    useDHCP = false;
    useHostResolvConf = false;
  };

  systemd.network = {
    enable = true;
    networks."50-eth0" = {
      matchConfig.Name = "eth0";
      networkConfig = {
        DHCP = "ipv4";
        IPv6AcceptRA = true;
      };
      linkConfig.RequiredForOnline = "routable";
    };
  };
}
