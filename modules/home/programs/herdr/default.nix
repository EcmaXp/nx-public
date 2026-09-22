{
  config,
  lib,
  pkgs,
  ...
}:
let
  # herdr installs ~/.claude/hooks/herdr-agent-state.sh and machine-reads only
  # the version marker in its header, so the body can be ours as long as the
  # header stays byte-identical. Keeping it in the store also makes it
  # immutable: `herdr integration install claude` can no longer overwrite it.
  agentStateHook = pkgs.writeTextFile {
    name = "herdr-agent-state.sh";
    executable = true;
    text = ''
      #!/bin/sh
      # installed by herdr
      # managed by herdr; reinstalling or updating the integration overwrites this file.
      # add custom hooks beside this file instead of editing it.
      # HERDR_INTEGRATION_ID=claude
      # HERDR_INTEGRATION_VERSION=10
      # NX_SHIM=herdr-agent-state 1 (patched from herdr claude asset v10)

      set -eu

      [ "''${1:-}" = "session" ] || exit 0
      [ "''${HERDR_ENV:-}" = "1" ] || exit 0
      [ -n "''${HERDR_SOCKET_PATH:-}" ] || exit 0
      [ -n "''${HERDR_PANE_ID:-}" ] || exit 0

      bin="''${HERDR_AGENT_STATE_BIN:-${lib.getExe pkgs.nx.herdr-agent-state}}"
      if [ ! -x "$bin" ]; then
        bin="$(command -v herdr-agent-state 2>/dev/null || true)"
      fi
      [ -n "$bin" ] && [ -x "$bin" ] || exit 0

      exec "$bin" claude "$@"
    '';
  };
in
lib.nx.gate config.nx.home.programs.herdr {
  home.packages = [ pkgs.nx.herdr ];

  home.symlink = {
    ".config/herdr/config.toml" = ./config.toml;
    ".config/herdr/ghostty.conf" = ./ghostty.conf;
  };

  # force: herdr wrote this path itself before nix took it over
  home.file = {
    ".claude/hooks/herdr-agent-state.sh" = {
      source = agentStateHook;
      force = true;
    };
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
          open -na Ghostty.app --env OP_CACHE_SESSION_KEY=(openssl rand -hex 32) --args \
              --config-file=$HOME/.config/herdr/ghostty.conf \
              --command=direct:(command -v herdr)
        '';
      };
    };
  };
}
