{ config, lib, ... }:
{
  config =
    lib.mkIf (config.nx.home.programs.claude.code.enable || config.nx.home.programs.codex.enable)
      {
        home.symlink = {
          ".agents/docs" = ../../../../docs/agents;
        };
      };
}
