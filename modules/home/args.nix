{
  config,
  host,
  inputs,
  pkgs,
  ...
}:
{
  _module.args = {
    osConfig =
      if config.nx.home.standalone.enable then
        null
      else
        (
          if pkgs.stdenv.hostPlatform.isDarwin then
            inputs.self.darwinConfigurations
          else
            inputs.self.nixosConfigurations
        ).${host}.config;
  };
}
