{ username, platform, ... }:
{
  imports = [
    ../bases
    ./default.nix
  ];

  home.username = username;
  home.homeDirectory =
    {
      nixos = "/home/${username}";
      darwin = "/Users/${username}";
    }
    .${platform};
}
