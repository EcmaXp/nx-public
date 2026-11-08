{
  config,
  lib,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.programs.herdr {
  home.packages = [ pkgs.herdr ];
}
