{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.claude.code {
  home.symlink = {
    ".claude/CLAUDE.md" = ./CLAUDE.md;
    ".claude/docs" = ./docs;
    ".claude/settings.json" = ./settings.json;
    ".claude/hooks/nx" = ./hooks;
  };

  programs.zsh.envExtra = ''
    [[ -n $CLAUDECODE && -f ~/.claude/hooks/nx/shell-env.sh ]] && source ~/.claude/hooks/nx/shell-env.sh
  '';
}
