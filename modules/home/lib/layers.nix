{
  config,
  lib,
  inputs,
  ...
}:
let
  mkRelativePath = lib.nx.mkRelativePath inputs.self config.home.homeDirectory;

  layerType = lib.types.submodule {
    options = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = true;
      };

      # Lower loads first. Consumers resolve a collision in favour of the first
      # definition, so 0 is reserved for whichever layer must never lose one,
      # and the assertion below keeps it unique.
      order = lib.mkOption {
        type = lib.types.ints.unsigned;
        default = 100;
      };

      # A directory inside this flake, rewritten to the live checkout so an
      # edit inside it applies without a switch.
      source = lib.mkOption {
        type = lib.types.nullOr lib.types.path;
        default = null;
      };

      # An absolute directory, used verbatim. This is the only way a store path
      # may enter a manifest: mkRelativePath leaves a non-flake path untouched
      # and then prepends the checkout to it, so a store path passed as `source`
      # silently becomes ~/nx/nix/store/... and the layer vanishes.
      dir = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
      };
    };
  };

  enabled =
    layers:
    lib.filter (entry: entry.layer.enable) (
      lib.mapAttrsToList (name: layer: { inherit name layer; }) layers
    );

  # A total order, so the manifest is byte-reproducible and a switch does not
  # churn it. Names break ties because attribute order alone is an accident.
  sorted =
    layers:
    lib.sort (
      a: b: if a.layer.order != b.layer.order then a.layer.order < b.layer.order else a.name < b.name
    ) (enabled layers);

  resolve =
    entry: if entry.layer.source != null then mkRelativePath entry.layer.source else entry.layer.dir;

  layerAssertions = kind: name: layer: [
    {
      assertion = (layer.source == null) != (layer.dir == null);
      message = "home.layers.${kind}.${name}: set exactly one of source or dir.";
    }
    {
      # A path literal inside the flake is a nix path value; an interpolated
      # store path is a string. That is the discriminator, and getting it
      # wrong erases the layer rather than failing loudly.
      assertion = layer.source == null || builtins.isPath layer.source;
      message = "home.layers.${kind}.${name}: source must be a path literal in this flake; a store path belongs in dir.";
    }
    {
      assertion = layer.dir == null || lib.hasPrefix "/" layer.dir;
      message = "home.layers.${kind}.${name}: dir must be absolute.";
    }
  ];

  reservedOrderAssertion = kind: layers: {
    assertion = lib.length (lib.filter (entry: entry.layer.order == 0) (enabled layers)) <= 1;
    message = "home.layers.${kind}: order 0 is reserved for the layer that must win every collision, so only one layer may claim it.";
  };
in
{
  options.home.layers = lib.mkOption {
    type = lib.types.attrsOf (lib.types.attrsOf layerType);
    default = { };
    description = ''
      Ordered directory layers, grouped by kind.

      Each kind materialises a manifest at `~/.nx/manifest/layers/<kind>`: one
      absolute directory per line, in load order. Every line is always read;
      nothing shadows anything, so there is no symlink farm and no activation
      step. A consumer that wants precedence has to define it over the merged
      contents, not by hiding a file.
    '';
  };

  config = {
    assertions = lib.concatLists (
      lib.mapAttrsToList (
        kind: layers:
        [ (reservedOrderAssertion kind layers) ]
        ++ lib.concatLists (lib.mapAttrsToList (layerAssertions kind) layers)
      ) config.home.layers
    );

    home.file = lib.mapAttrs' (
      kind: layers:
      lib.nameValuePair ".nx/manifest/layers/${kind}" {
        text = lib.concatMapStrings (entry: resolve entry + "\n") (sorted layers);
      }
    ) config.home.layers;
  };
}
