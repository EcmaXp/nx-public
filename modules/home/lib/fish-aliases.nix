{
  config,
  lib,
  pkgs,
  ...
}:
let
  aliases = config.programs.fish.lazyAliases;
  aliasFunctions =
    pkgs.runCommand "fish-alias-functions" { nativeBuildInputs = [ config.programs.fish.package ]; }
      ''
        mkdir $out
        export HOME=$TMPDIR
        ${lib.concatStrings (
          lib.mapAttrsToList (name: body: ''
            command fish --no-config -c ${lib.escapeShellArg "alias ${lib.escapeShellArg name} ${lib.escapeShellArg body}; functions ${lib.escapeShellArg name}"} > $out/${name}.fish
            grep -q '^function ${name} ' $out/${name}.fish
          '') aliases
        )}
      '';
in
{
  options.programs.fish.lazyAliases = lib.mkOption {
    type = lib.types.attrsOf lib.types.str;
    default = { };
  };

  config = lib.mkIf (aliases != { }) {
    xdg.configFile = lib.mapAttrs' (
      name: _:
      lib.nameValuePair "fish/functions/${name}.fish" { source = "${aliasFunctions}/${name}.fish"; }
    ) aliases;
  };
}
