{
  platform,
  pkgs,
  osConfig ? { },
  ...
}:
let
  homebrewPrefix = osConfig.homebrew.prefix or "/opt/homebrew";
in
{
  config.lib.brew = {
    prefix = homebrewPrefix;
    wrapPackage =
      { name, pkg }:
      if platform == "darwin" then
        pkgs.runCommand name { meta.mainProgram = name; } ''
          mkdir -p $out/bin
          ln -s ${homebrewPrefix}/bin/${name} $out/bin/${name}
        ''
      else
        pkg;
  };
}
