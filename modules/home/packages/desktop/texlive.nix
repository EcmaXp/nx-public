{
  config,
  lib,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.packages.desktop {
  home.packages = [
    (pkgs.texliveSmall.withPackages (
      ps: with ps; [
        framed
        xelatex-dev
      ]
    ))
  ];
}
