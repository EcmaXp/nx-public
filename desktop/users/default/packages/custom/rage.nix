{
  inputs,
  pkgs,
  lib,
  ...
}:
let
  age-plugin-op = pkgs.callPackage ./age-plugin-op.nix { };

  age-plugins = [
    age-plugin-op
    pkgs.age-plugin-se
    pkgs.age-plugin-yubikey
  ];

  ragenix = inputs.ragenix.packages.${pkgs.stdenv.hostPlatform.system}.default.override {
    plugins = age-plugins;
  };

  age-wrapper = name: final: prev: {
    ${name} = prev.${name}.overrideAttrs (old: {
      nativeBuildInputs = old.nativeBuildInputs ++ [
        prev.makeWrapper
      ];

      postFixup = ''
        wrapProgram "$out/bin/${old.meta.mainProgram}" \
          --suffix PATH : ${lib.strings.makeBinPath age-plugins}
      '';
    });
  };
in
{
  home.packages = [
    age-plugin-op
    pkgs.age-plugin-se
    ragenix
  ];

  nixpkgs.overlays = [
    (age-wrapper "age")
    (age-wrapper "rage")
  ];

  age = {
    package = ragenix;
  };
}
