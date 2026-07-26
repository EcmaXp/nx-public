{ config, lib, ... }:
{
  imports = [
    ../lib/desktop.nix
    ../programs/git/desktop.nix
    ../secrets/age.nix
    ../shells/desktop.nix
  ];

  config = lib.mkIf config.nx.roles.desktop.enable {
    nx.home = {
      lib.mise.enable = lib.mkDefault true;
      packages.desktop.enable = lib.mkDefault true;
      programs = {
        aerospace.enable = lib.mkDefault true;
        claude = {
          enable = lib.mkDefault true;
          code.enable = lib.mkDefault true;
        };
        ctx.enable = lib.mkDefault true;
        ghostty.enable = lib.mkDefault true;
        git.desktop.enable = lib.mkDefault true;
        gnupg.enable = lib.mkDefault true;
        herdr.enable = lib.mkDefault true;
        man.enable = lib.mkDefault true;
        mise.enable = lib.mkDefault true;
        obsidian.enable = lib.mkDefault true;
        onepassword.enable = lib.mkDefault true;
        pyp.enable = lib.mkDefault true;
        sem.enable = lib.mkDefault true;
        terraform.enable = lib.mkDefault true;
        uv.enable = lib.mkDefault true;
        zed.enable = lib.mkDefault true;
      };
      scripts.desktop.enable = lib.mkDefault true;
      secrets.age.enable = lib.mkDefault true;
      shells = {
        desktop.enable = lib.mkDefault true;
        aliases = {
          desktop.enable = lib.mkDefault true;
          kubectl.enable = lib.mkDefault true;
        };
        plugins = {
          enable = lib.mkDefault true;
          atuin.enable = lib.mkDefault true;
          direnv.enable = lib.mkDefault true;
          granted.enable = lib.mkDefault true;
          kubeswitch.enable = lib.mkDefault true;
          safe-rm.enable = lib.mkDefault true;
          starship.enable = lib.mkDefault true;
          tfswitch.enable = lib.mkDefault true;
        };
        shortcuts.enable = lib.mkDefault true;
      };
    };
  };
}
