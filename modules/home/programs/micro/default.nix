{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.micro {
  home.sessionVariables = {
    EDITOR = "micro";
  };
}
