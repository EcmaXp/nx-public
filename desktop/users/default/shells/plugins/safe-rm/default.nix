{
  # https://github.com/NixOS/nixpkgs/blob/master/pkgs/tools/system/safe-rm/default.nix
  # https://nixos.wiki/wiki/Overlays
  nixpkgs.overlays = [
    (final: prev: {
      safe-rm = prev.safe-rm.overrideAttrs (oldAttrs: {
        postPatch = ''
          substituteInPlace src/main.rs \
            --replace "/bin/rm" "/run/current-system/sw/bin/rm"
        '';
      });
    })
  ];
}
