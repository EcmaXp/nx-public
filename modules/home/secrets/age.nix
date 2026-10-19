{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:
let
  inherit (config.home) homeDirectory;
  agePlugins = [
    pkgs.nx.age-plugin-op
    pkgs.age-plugin-se
    pkgs.age-plugin-yubikey
  ];
  ragenix = inputs.ragenix.packages.${pkgs.stdenv.hostPlatform.system}.default.override {
    plugins = agePlugins;
  };
in
lib.nx.gate config.nx.home.secrets.age {
  home.packages = [
    pkgs.nx.age-plugin-op
    pkgs.age-plugin-se
    ragenix
  ];

  age = {
    package = ragenix;
    identityPaths = lib.mkBefore [
      "${homeDirectory}/.ssh/id_machine"
    ];
  };
}
