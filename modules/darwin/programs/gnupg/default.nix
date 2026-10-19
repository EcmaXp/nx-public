{ config, lib, ... }:
lib.nx.gate config.nx.darwin.programs.gnupg {
  programs.gnupg.agent = {
    enable = true;
  };
}
