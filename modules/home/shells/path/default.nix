{ config, lib, ... }:
let
  last = lib.mkOrder 10000;
  bashSource = "source ${./deduplicate.bash}";
  zshSource = "source ${./deduplicate.zsh}";
  nushellDir = config.programs.nushell.configDir;
in
lib.nx.gate config.nx.home.shells.path {
  programs.bash = {
    bashrcExtra = last ''
      if [[ $- != *i* ]]; then
        ${bashSource}
      fi
    '';
    initExtra = last bashSource;
  };

  programs.fish = {
    shellInitLast = last "source ${./deduplicate.fish}";
  };

  programs.zsh = {
    envExtra = last ''
      if [[ ! -o interactive && ! -o login ]]; then
        ${zshSource}
      fi
    '';
    initContent = last ''
      if [[ ! -o login ]]; then
        ${zshSource}
      fi
    '';
    loginExtra = last zshSource;
  };

  home.file = lib.mkIf config.programs.nushell.enable {
    "${nushellDir}/autoload/zz-nx-path-deduplicate.nu" = {
      source = ./deduplicate.nu;
    };
  };
}
