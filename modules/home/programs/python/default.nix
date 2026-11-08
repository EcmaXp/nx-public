{
  config,
  inputs,
  lib,
  ...
}:
let
  inherit (config.home) homeDirectory;
  inherit (inputs.home-manager.lib) hm;
  pycachePrefix = "${homeDirectory}/.cache/python";
in
lib.nx.gate config.nx.home.programs.python {
  home.sessionVariables = {
    PYTHONPYCACHEPREFIX = pycachePrefix;
  };

  home.activation.clearProfilePycache = hm.dag.entryAfter [ "writeBoundary" ] ''
    run rm -rf \
      ${pycachePrefix}${homeDirectory}/.nix-profile \
      ${pycachePrefix}${homeDirectory}/.local/state/nix/profiles \
      ${pycachePrefix}/etc/profiles \
      ${pycachePrefix}/run
  '';
}
