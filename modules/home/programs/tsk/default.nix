{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:
let
  inherit (inputs.home-manager.lib) hm;
  cfg = config.nx.home.programs;
  herdr = lib.getExe pkgs.nx.herdr;
  jq = lib.getExe pkgs.jq;
  pluginLink = "${config.home.homeDirectory}/.config/herdr/tsk-plugin";

  herdrPlugin = pkgs.runCommand "herdr-tsk-plugin" { } ''
    cp -r ${./herdr-plugin} $out
    chmod -R u+w $out
    substituteInPlace $out/herdr-plugin.toml $out/scripts/open-board.sh \
      --replace-fail @tsk@ ${config.home.homeDirectory}/.local/share/mise/shims/tsk
  '';
in
lib.nx.gate cfg.tsk {
  home.sessionVariables = {
    TSK_NO_UPDATE_CHECK = "1";
  };

  home.symlink = {
    ".agents/skills/tsk-cli/SKILL.md" = ../claude/code/plugins/nx/skills/tsk-cli/SKILL.md;
  };

  home.file = lib.mkIf cfg.herdr.enable {
    ".config/herdr/tsk-plugin" = {
      source = herdrPlugin;
    };
  };

  home.activation = lib.mkIf cfg.herdr.enable {
    linkHerdrTsk = hm.dag.entryAfter [ "writeBoundary" ] ''
      tskRoot="$(${herdr} plugin list --plugin herdr-tsk --json 2>/dev/null \
        | ${jq} -r '.result.plugins[0].plugin_root // empty' || true)"
      if [ "$tskRoot" != ${pluginLink} ] && [ "$tskRoot" != "$(readlink -f ${pluginLink})" ]; then
        if [ -n "$tskRoot" ]; then
          run ${herdr} plugin unlink herdr-tsk || true
        fi
        run ${herdr} plugin link ${pluginLink} \
          || warnEcho "herdr-tsk: link failed; run: herdr plugin link ${pluginLink}"
      fi
    '';
  };
}
