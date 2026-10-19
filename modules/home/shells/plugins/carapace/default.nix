{
  config,
  lib,
  pkgs,
  ...
}:
let
  carapaceInit =
    pkgs.runCommand "carapace-init" { nativeBuildInputs = [ config.programs.carapace.package ]; }
      ''
        HOME=/homeless-shelter command carapace _carapace fish | sed "s|'/homeless-shelter|\"\$HOME\"'|g" > full.fish
        ! grep -q '${builtins.storeDir}' full.fish
        ! grep -q /homeless-shelter full.fish

        mkdir -p $out/completions
        grep -v '^complete -[ec] "' full.fish > $out/init.fish
        grep '^complete -[ec] "' full.fish | while IFS= read -r line; do
          name=''${line#complete -? \"}
          name=''${name%%\"*}
          printf '%s\n' "$line" >> "$out/completions/$name.fish"
        done

        grep -q '^function _carapace_completer' $out/init.fish
        test "$(ls $out/completions | wc -l)" = "$(grep -c '^complete -e "' full.fish)"
      '';
in
lib.nx.gate config.nx.home.shells.plugins.carapace {
  programs.carapace = {
    enable = true;
    enableFishIntegration = false;
  };

  programs.fish = {
    interactiveShellInit = ''
      set -p fish_complete_path ${carapaceInit}/completions
      source ${carapaceInit}/init.fish
    '';
  };
}
