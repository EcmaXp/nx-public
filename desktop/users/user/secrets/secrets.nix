let
  identities = import ./identities "null";
  publicKeys = identities.publicKeys;
  default = { inherit publicKeys; };
in
{
  "example.yaml.age" = default;
}
