{
  imports = [
    ./nx
  ];

  home.scripts = {
    default = {
      bin = ./bin;
      fish = ./fish;
    };
  };
}
