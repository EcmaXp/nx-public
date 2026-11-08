{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.atuin;
  atuinInit =
    pkgs.runCommand "atuin-init.fish"
      {
        nativeBuildInputs = [
          cfg.package
          pkgs.writableTmpDirAsHomeHook
        ];
      }
      ''
        command atuin init fish ${lib.escapeShellArgs cfg.flags} > init.fish
        grep -q '(atuin uuid)' init.fish
        grep -q '^function _atuin_preexec --on-event fish_preexec$' init.fish
        grep -q '^function _atuin_postexec --on-event fish_postexec$' init.fish
        sed \
          -e 's|(atuin uuid)|(printf %04x (random 0 65535) (random 0 65535) (random 0 65535) (random 0 65535) (random 0 65535) (random 0 65535) (random 0 65535) (random 0 65535))|' \
          -e '/^function _atuin_preexec /,/^end$/d' \
          -e '/^function _atuin_postexec /,/^end$/d' \
          init.fish > $out
        cat ${./history.fish} >> $out
        ! grep -q 'atuin uuid' $out
        ! grep -q '(atuin history start' $out
        test "$(grep -c '^function _atuin_p[a-z]*exec ' $out)" = 2
        ! grep -q '${builtins.storeDir}' $out
      '';
in
lib.nx.gate config.nx.home.shells.plugins.atuin {
  programs.atuin = {
    enable = true;
    enableFishIntegration = false;
    flags = [ "--disable-up-arrow" ];
    settings = {
      style = "compact";
      inline_height = 40;
      enter_accept = false;
    };
  };

  programs.fish = {
    interactiveShellInit = ''
      source ${atuinInit}
    '';
  };
}
