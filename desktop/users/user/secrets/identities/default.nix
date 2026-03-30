hostName:
let
  publicMachineIdentityPaths = [
    ./host.se.pub
  ];
  onePasswordIdentityPaths = [
    ./user-example.op
  ];
  publicOnePasswordIdentityPaths = [
    ./user-example.op.pub
  ];
  yubikeyIdentityPaths = [
    ./yubikey-example.yubikey
  ];
  publicyYubikeyIdentityPaths = [
    ./yubikey-example.ssh.pub
    ./yubikey-example.yubikey.pub
  ];
  identityPaths = (onePasswordIdentityPaths ++ yubikeyIdentityPaths);
  publicIdentityPaths = (
    publicOnePasswordIdentityPaths ++ publicMachineIdentityPaths ++ publicyYubikeyIdentityPaths
  );
in
{
  identityPaths = identityPaths;
  publicKeys = (
    builtins.map (
      file: builtins.replaceStrings [ "\n" ] [ "" ] (builtins.readFile file)
    ) publicIdentityPaths
  );
}
