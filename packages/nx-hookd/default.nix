{ pkgs, lib, ... }:
let
  python = pkgs.python3.withPackages (_: [
    (pkgs.python3Packages.toPythonModule pkgs.nx.sidepulse)
  ]);
in
pkgs.runCommand "nx-hookd"
  {
    meta = {
      description = "Socket daemon that serves Claude Code telemetry hooks without a process per event";
      platforms = lib.platforms.darwin;
      mainProgram = "nx-hookd";
    };
  }
  ''
    mkdir -p $out/bin
    { echo '#!${python.interpreter}'; cat ${./nx_hookd.py}; } > $out/bin/nx-hookd
    chmod +x $out/bin/nx-hookd
  ''
