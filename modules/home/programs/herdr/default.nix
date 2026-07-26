{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.herdr {
  home.symlink = {
    ".config/herdr/config.toml" = ./config.toml;
    ".config/herdr/ghostty.conf" = ./ghostty.conf;
  };

  # bare `herdr` opens a dedicated ghostty instance whose config rebinds
  # cmd/ctrl chords to herdr prefix bytes; see ./ghostty.conf
  programs.fish = {
    functions = {
      herdr = {
        wraps = "herdr";
        body = ''
          if set -q HERDR_ENV; or test (count $argv) -gt 0
              command herdr $argv
              return
          end
          open -na Ghostty.app --args \
              --config-file=$HOME/.config/herdr/ghostty.conf \
              --command=direct:(command -v herdr)
        '';
      };
    };
  };
}
