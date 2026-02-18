{ lib, ... }:
{
  # Workaround for inetutils 2.7 build failure on Darwin
  nixpkgs.overlays = [
    (self: super: {
      inetutils = super.inetutils.overrideAttrs (old: {
        hardeningDisable =
          (old.hardeningDisable or [ ]) ++ lib.optional super.stdenv.hostPlatform.isDarwin "format";
      });
    })
  ];
}
