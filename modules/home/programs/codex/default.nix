{ config, lib, ... }:
let
  guidanceFiles = [
    ../../../../docs/agents/guides/user-preferences.md
    ../../../../docs/agents/guides/writing-style.md
    ../../../../docs/agents/guides/korean-style.md
    ../../../../docs/agents/guides/comment-style.md
    ../../../../docs/agents/tools/RTK.md
  ];
  guidance = builtins.concatStringsSep "\n\n" (map builtins.readFile guidanceFiles) + "\n";
in
lib.nx.gate config.nx.home.programs.codex {
  home.file = {
    ".codex/AGENTS.md" = {
      text = guidance;
      force = true;
    };
  };
}
