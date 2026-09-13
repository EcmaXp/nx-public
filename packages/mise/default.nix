{ lib, pkgs, ... }:
let
  version = "2026.9.1";
  assets = {
    aarch64-darwin = {
      name = "macos-arm64";
      hash = "sha256-v+oKtBe0jB6LmUEvyvIM4XQkoyhqh2bX0rAFH+Mh1WU=";
    };
    x86_64-linux = {
      name = "linux-x64-musl";
      hash = "sha256-sTvNi8UWdz/ZtgBojObmwBtiBxrlf2quRL4HE6bYNPE=";
    };
  };
  asset = assets.${pkgs.stdenv.hostPlatform.system};
in
pkgs.stdenvNoCC.mkDerivation {
  pname = "mise";
  inherit version;

  src = pkgs.fetchurl {
    url = "https://github.com/jdx/mise/releases/download/v${version}/mise-v${version}-${asset.name}.tar.gz";
    inherit (asset) hash;
  };

  # Stripping invalidates the ad-hoc code signature Apple's loader requires.
  dontStrip = true;

  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -r bin man share $out/
    runHook postInstall
  '';

  meta = {
    description = "Dev tools, env vars, and tasks in one CLI";
    homepage = "https://github.com/jdx/mise";
    license = lib.licenses.mit;
    mainProgram = "mise";
    platforms = lib.attrNames assets;
  };
}
