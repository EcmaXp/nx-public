{ config, lib, ... }:
let
  inherit (config.home) homeDirectory;
in
lib.nx.gate config.nx.home.programs.pyp {
  home.sessionVariables = {
    PYP_CONFIG_PATH = "${homeDirectory}/.config/pyp/pypcfg.py";
  };

  home.symlink = {
    ".config/pyp/pypcfg.py" = ./pypcfg.py;
  };
}
