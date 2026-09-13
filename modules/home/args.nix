{
  host,
  inputs,
  pkgs,
  ...
}:
{
  _module.args = {
    osConfig =
      (if pkgs.stdenv.isDarwin then inputs.self.darwinConfigurations else inputs.self.nixosConfigurations)
      .${host}.config;
  };
}
