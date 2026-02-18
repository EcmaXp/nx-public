{ pkgs, ... }:
{
  imports = [
    ./aliases
    ./plugins
    ./shortcuts
  ];

  home.sessionVariables = {
    RUNEWIDTH_EASTASIAN = "0";
  };

  programs.fish = {
    shellInit = ''
      /opt/homebrew/bin/brew shellenv fish | source
      for nix_profile in (string split " " $NIX_PROFILES)
        fish_add_path -gm $nix_profile/bin
      end
      fish_add_path -gm ~/.dotnet/tools
      fish_add_path -gm ~/.krew/bin
      fish_add_path -gm ~/.cargo/bin
      fish_add_path -gm ~/go/bin
      fish_add_path -gm ~/.local/bin
    '';
  };

  programs.fish = {
    functions = {
      # nix-command-not-found
      __fish_command_not_found_handler = {
        body = "~/.local/bin/nix-command-not-found $argv";
        onEvent = "fish_command_not_found";
      };
    };
  };

  programs.bash.enable = true;
  programs.zsh.enable = true;
  programs.nushell.enable = true;

  home.file = {
    ".local/bin/nix-command-not-found" = {
      executable = true;
      text = ''
        #!/usr/bin/env bash
        source ${pkgs.nix-index}/etc/profile.d/command-not-found.sh
        command_not_found_handle "$@"
      '';
    };
  };
}
