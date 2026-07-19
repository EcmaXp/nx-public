{ ... }:
{
  brew = {
    wrapPackage =
      {
        pkgs,
        prefix ? "/opt/homebrew",
        name,
        pkg ? null,
      }:
      if pkgs.stdenv.hostPlatform.isDarwin then
        pkgs.runCommand name { meta.mainProgram = name; } ''
          mkdir -p $out/bin
          ln -s ${prefix}/bin/${name} $out/bin/${name}
        ''
      else
        pkg;
  };
}
