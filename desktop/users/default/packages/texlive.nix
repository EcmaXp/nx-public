{ pkgs, ... }:
{
  home.packages = [
    (pkgs.texliveSmall.withPackages (
      ps: with ps; [
        framed
        xelatex-dev
      ]
    ))
  ];
}
