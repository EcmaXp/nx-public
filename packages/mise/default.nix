{ lib, pkgs, ... }:
let
  version = "2026.11.0";
  assets = {
    aarch64-darwin = {
      name = "macos-arm64";
      hash = "sha256-nDZ4sgUV8JEgizoqmQC7xat/IoocWJOEnw8pObGCnfM=";
    };
    aarch64-linux = {
      name = "linux-arm64-musl";
      hash = "sha256-9R3CSilbooGg/ZYm3YyvKxbYN8TZviQS2GyTPmDCAPg=";
    };
    x86_64-linux = {
      name = "linux-x64-musl";
      hash = "sha256-TloICr560DKtLOZF76mbM5n9nIURt6O9qqb3HlRfKRo=";
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
