{
  config,
  lib,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.programs.claude.code {
  home.packages = [
    pkgs.nx.nx-bash-safe
    pkgs.nx.nx-claude-statusline
    pkgs.nx.nx-comment-scan
    pkgs.nx.nx-vibe-island
  ];

  home.symlink = {
    ".claude/CLAUDE.md" = ./CLAUDE.md;
    ".claude/settings.json" = ./settings.json;
    ".claude/hooks/nx" = ./hooks;
  };

  # The lowest policy layer, and the one that must win every name collision:
  # it is the read surface of third-party tools, which no later layer may
  # restate. `dir` rather than `source` because a store path must be used
  # verbatim. Order 0 is reserved for exactly this.
  home.layers.bash-safe.baseline = {
    order = 0;
    dir = "${pkgs.nx.nx-bash-safe}/share/nx-bash-safe/policy";
  };

  programs.zsh.envExtra = ''
    [[ -n $CLAUDECODE && -f ~/.claude/hooks/nx/shell-env.sh ]] && source ~/.claude/hooks/nx/shell-env.sh
  '';
}
