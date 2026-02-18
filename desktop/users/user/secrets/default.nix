{ hostname, ... }:
let
  identities = import ./identities hostname;
in
{
  age = {
    identityPaths = identities.identityPaths;
    secrets = {
      "example.yaml".file = ./example.yaml.age;
    };
  };
}
