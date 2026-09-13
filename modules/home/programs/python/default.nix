{ config, lib, ... }:
let
  inherit (config.home) homeDirectory;
in
lib.nx.gate config.nx.home.programs.python {
  home.sessionVariables = {
    PYTHONPYCACHEPREFIX = "${homeDirectory}/.cache/python";
  };
}
