{
  config,
  lib,
  ...
}:
lib.nx.gate config.nx.home.programs.tsk {
  home.sessionVariables = {
    TSK_NO_UPDATE_CHECK = "1";
  };

  home.symlink = {
    ".agents/skills/tsk-cli/SKILL.md" = ../claude/code/plugins/nx/skills/tsk-cli/SKILL.md;
  };
}
