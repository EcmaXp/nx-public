let
  openssh = self: super: {
    openssh = super.openssh.override {
      withKerberos = true;
    };
  };
in
{
  nixpkgs.overlays = [
    openssh
  ];
}
