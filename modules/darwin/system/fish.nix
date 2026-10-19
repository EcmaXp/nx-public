{
  config,
  lib,
  pkgs,
  ...
}:
let
  shellInit = config.environment.interactiveShellInit;
  nixIndexHook = "source ${config.programs.nix-index.package}/etc/profile.d/command-not-found.sh";
in
{
  config = lib.mkIf config.programs.nix-index.enable {
    assertions = [
      {
        assertion = lib.hasInfix (builtins.unsafeDiscardStringContext nixIndexHook) (
          builtins.unsafeDiscardStringContext shellInit
        );
        message = "nix-darwin no longer sources the nix-index hook as expected";
      }
    ];

    environment.etc."fish/interactiveShellInit.fish".source = lib.mkForce (
      pkgs.runCommand "interactiveShellInit.fish"
        { nativeBuildInputs = [ config.programs.fish.babelfishPackage ]; }
        ''
          command babelfish < ${
            pkgs.writeText "interactiveShellInit" (lib.replaceStrings [ nixIndexHook ] [ "" ] shellInit)
          } > $out
          ! grep -q command-not-found $out
        ''
    );
  };
}
