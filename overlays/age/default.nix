{ lib, ... }:
let
  ageWrapper = name: final: prev: {
    ${name} = prev.${name}.overrideAttrs (old: {
      nativeBuildInputs = old.nativeBuildInputs ++ [
        prev.makeWrapper
      ];

      postFixup = ''
        wrapProgram "$out/bin/${old.meta.mainProgram}" \
          --suffix PATH : ${
            lib.strings.makeBinPath [
              final.nx.age-plugin-op
              final.age-plugin-se
              final.age-plugin-yubikey
            ]
          }
      '';
    });
  };
in
final: prev: (ageWrapper "age" final prev) // (ageWrapper "rage" final prev)
