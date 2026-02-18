{ config, ... }:
let
  inherit (config.home) homeDirectory;
in
{
  home.sessionVariables = {
    PYP_CONFIG_PATH = "${homeDirectory}/.config/pyp/pypcfg.py";
  };

  home.symlink = {
    ".config/pyp/pypcfg.py" = ./pypcfg.py;
  };
}
