{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.locale {
  home.sessionVariables = {
    RUNEWIDTH_EASTASIAN = "0";
  };
}
