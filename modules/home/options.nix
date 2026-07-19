{ lib, ... }:
let
  enable = description: lib.mkEnableOption description;
  userOptions = lib.types.attrsOf (
    lib.types.submodule {
      options.enable = lib.mkEnableOption "this user profile";
    }
  );
in
{
  options.nx = {
    roles = {
      desktop.enable = enable "Home Manager desktop profile";
      server.enable = enable "Home Manager server profile";
    };

    users = lib.mkOption {
      type = userOptions;
      default = { };
    };

    home = {
      lib.mise.enable = enable "mise helper module";
      packages.desktop.enable = enable "desktop package set";
      programs = {
        aerospace.enable = enable "AeroSpace configuration";
        claude = {
          enable = enable "Claude program configuration";
          code.enable = enable "Claude Code configuration";
        };
        ctx.enable = enable "ctx configuration";
        ghostty.enable = enable "Ghostty configuration";
        git.desktop.enable = enable "desktop Git configuration";
        gnupg.enable = enable "GnuPG configuration";
        herdr.enable = enable "herdr configuration";
        man.enable = enable "manual page configuration";
        mise.enable = enable "mise configuration";
        obsidian.enable = enable "Obsidian configuration";
        onepassword.enable = enable "1Password SSH agent integration";
        pyp.enable = enable "pyp configuration";
        sem.enable = enable "sem (sem-cli) via brew";
        terraform.enable = enable "Terraform configuration";
        uv.enable = enable "uv configuration";
        zed.enable = enable "Zed configuration";
      };
      scripts = {
        desktop.enable = enable "desktop scripts";
        users = lib.mkOption {
          type = userOptions;
          default = { };
        };
      };
      secrets = {
        age.enable = enable "desktop age and ragenix setup";
        users = lib.mkOption {
          type = userOptions;
          default = { };
        };
      };
      server.enable = enable "server home profile";
      shells = {
        desktop.enable = enable "desktop shell defaults";
        aliases = {
          desktop.enable = enable "desktop shell aliases";
          kubectl.enable = enable "kubectl shell aliases";
        };
        plugins = {
          enable = enable "desktop shell plugin defaults";
          atuin.enable = enable "atuin shell integration";
          direnv.enable = enable "direnv shell integration";
          granted.enable = enable "granted shell integration";
          kubeswitch.enable = enable "kubeswitch shell integration";
          safe-rm.enable = enable "safe-rm shell integration";
          starship.enable = enable "starship shell integration";
          tfswitch.enable = enable "tfswitch shell integration";
        };
        shortcuts.enable = enable "shell shortcuts";
      };
      users = lib.mkOption {
        type = userOptions;
        default = { };
      };
    };
  };
}
