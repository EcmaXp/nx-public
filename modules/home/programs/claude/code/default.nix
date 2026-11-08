{
  config,
  lib,
  pkgs,
  ...
}:
let
  sandboxWritableDirs = map (
    entry:
    if lib.hasPrefix "~/" entry then
      "${config.home.homeDirectory}/${lib.removePrefix "~/" entry}"
    else
      entry
  ) (lib.importJSON ./settings.json).sandbox.filesystem.allowWrite;
  sandboxCanWrite =
    path: lib.any (dir: path == dir || lib.hasPrefix "${dir}/" path) sandboxWritableDirs;
  programOf =
    agent:
    if agent.config.Program != null then
      toString agent.config.Program
    else
      lib.concatStrings (lib.take 1 (lib.defaultTo [ ] agent.config.ProgramArguments));
in
lib.nx.gate config.nx.home.programs.claude.code {
  home.packages = [
    pkgs.nx.nx-bash-safe
    pkgs.nx.nx-claude-statusline
    pkgs.nx.nx-comment-scan
  ];

  home.symlink = {
    ".claude/CLAUDE.md" = ./CLAUDE.md;
    ".claude/settings.json" = ./settings.json;
    ".claude/keybindings.json" = ./keybindings.json;
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

  launchd.agents = lib.mkIf pkgs.stdenv.isDarwin {
    nx-hookd = {
      enable = true;
      config = {
        ProgramArguments = [ (lib.getExe pkgs.nx.nx-hookd) ];
        KeepAlive = true;
        RunAtLoad = true;
        ProcessType = "Background";
        StandardErrorPath = "${config.home.homeDirectory}/.local/state/nx/hookd.log";
      };
    };
  };

  assertions = lib.mapAttrsToList (name: agent: {
    assertion = !agent.enable || !sandboxCanWrite (programOf agent);
    message = "launchd agent ${name} runs ${programOf agent}, which sandboxed Claude Code can rewrite";
  }) config.launchd.agents;

  programs.zsh.envExtra = ''
    [[ -n $CLAUDECODE && -f ~/.claude/hooks/nx/shell-env.sh ]] && source ~/.claude/hooks/nx/shell-env.sh
  '';
}
