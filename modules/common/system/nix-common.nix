{ pkgs, ... }:
{
  nix = {
    package = pkgs.nix;
    settings = {
      experimental-features = "nix-command flakes";
    };
  };

  programs.nix-index.enable = true;
  programs.nix-index-database.comma.enable = true;
}
