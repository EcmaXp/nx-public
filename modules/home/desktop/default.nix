{ config, lib, ... }:
{
  imports = [
    ../programs/git/desktop.nix
    ../secrets/age.nix
    ../shells/desktop.nix
  ];

  config = lib.mkIf config.nx.roles.desktop.enable {
    nx.home = {
      packages.desktop.enable = lib.mkDefault true;
      programs = {
        aerospace.enable = lib.mkDefault true;
        aws.enable = lib.mkDefault true;
        claude = {
          enable = lib.mkDefault true;
          code.enable = lib.mkDefault true;
        };
        codex.enable = lib.mkDefault true;
        ghostty.enable = lib.mkDefault true;
        git.desktop.enable = lib.mkDefault true;
        gnupg.enable = lib.mkDefault true;
        gomi.enable = lib.mkDefault true;
        herdr.enable = lib.mkDefault true;
        locale.enable = lib.mkDefault true;
        man.enable = lib.mkDefault true;
        micro.enable = lib.mkDefault true;
        onepassword.enable = lib.mkDefault true;
        pyp.enable = lib.mkDefault true;
        python.enable = lib.mkDefault true;
        terraform.enable = lib.mkDefault true;
        tsk.enable = lib.mkDefault true;
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
        path.enable = lib.mkDefault true;
        plugins = {
          atuin.enable = lib.mkDefault true;
          broot.enable = lib.mkDefault true;
          carapace.enable = lib.mkDefault true;
          direnv.enable = lib.mkDefault true;
          eza.enable = lib.mkDefault true;
          fzf.enable = lib.mkDefault true;
          granted.enable = lib.mkDefault true;
          kubeswitch.enable = lib.mkDefault true;
          navi.enable = lib.mkDefault true;
          safe-rm.enable = lib.mkDefault true;
          skim.enable = lib.mkDefault true;
          starship.enable = lib.mkDefault true;
          tfswitch.enable = lib.mkDefault true;
          tmux.enable = lib.mkDefault true;
          watson.enable = lib.mkDefault true;
          yazi.enable = lib.mkDefault true;
          zellij.enable = lib.mkDefault true;
          zoxide.enable = lib.mkDefault true;
        };
        shortcuts.enable = lib.mkDefault true;
      };
    };
    programs.mise.enable = lib.mkDefault true;
  };
}
