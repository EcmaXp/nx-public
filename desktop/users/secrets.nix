{
  config,
  inputs,
  lib,
  ...
}:
let
  inherit (config.home) homeDirectory;
in
{
  imports = [
    inputs.ragenix.homeManagerModules.default
  ];

  age.identityPaths = lib.mkBefore [
    "${homeDirectory}/.ssh/id_machine"
  ];
}
