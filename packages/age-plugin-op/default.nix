{ pkgs, lib, ... }:
pkgs.buildGoModule rec {
  pname = "age-plugin-op";
  version = "0.1.0";

  src = pkgs.fetchFromGitHub {
    owner = "bromanko";
    repo = "age-plugin-op";
    rev = "v${version}";
    hash = "sha256-1fzAdA2x26Q3N/2Myxtbky7eddJZod4ctbuiSNVX5ig=";
  };

  vendorHash = "sha256-dhJdLYy/CDqZuF5/1v05/ZEp+cWJ6V4GnVCf+mUr1MU=";

  env.CGO_ENABLED = "0";

  meta = with lib; {
    description = "An age client plugin using 1Password sourced keys";
    homepage = "https://github.com/bromanko/age-plugin-op";
    license = licenses.mit;
    mainProgram = "age-plugin-op";
  };
}
