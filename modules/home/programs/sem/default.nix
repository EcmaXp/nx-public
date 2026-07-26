{
  config,
  lib,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.programs.sem {
  home.packages = [
    (lib.nx.brew.wrapPackage {
      inherit pkgs;
      name = "sem";
    })
  ];
}
