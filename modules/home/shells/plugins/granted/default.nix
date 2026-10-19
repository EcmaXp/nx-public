{
  config,
  lib,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.shells.plugins.granted {
  programs.fish = {
    lazyAliases = {
      "assume" = "source ${pkgs.granted}/share/assume.fish";
    };
  };
}
