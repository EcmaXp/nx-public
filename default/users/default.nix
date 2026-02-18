{
  imports = [
    ./default
  ];

  home.stateVersion = "24.11";

  programs.home-manager.enable = true;

  nixpkgs = {
    config = {
      allowBroken = true;
      allowUnfree = true;
    };
  };
}
