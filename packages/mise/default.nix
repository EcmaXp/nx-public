{ lib, pkgs, ... }:
let
  version = "2026.9.12";
  assets = {
    aarch64-darwin = {
      name = "macos-arm64";
      hash = "sha256-Dxx/PnTYya6Cl25pkAWPK8aIIazGtMN6aMIGyDZpJBk=";
    };
    aarch64-linux = {
      name = "linux-arm64-musl";
      hash = "sha256-32ldvPpUN5IIG9uG+Z/5HD3FuQQ5YXGzCLbvHhVzAYI=";
    };
    x86_64-linux = {
      name = "linux-x64-musl";
      hash = "sha256-GyBR2NPvq02chc61mEmj6wUtskrUBqOQH9a0U3TFbr4=";
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
