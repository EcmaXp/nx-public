{
  imports = [
    ./nx.nix
  ];

  home.scripts = {
    default = {
      bin = ../../../../tools/nx/bin/core;
      fish = ../../../../tools/nx/fish;
    };
  };
}
