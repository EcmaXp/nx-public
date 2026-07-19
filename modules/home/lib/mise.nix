{
  config,
  lib,
  ...
}:
let
  toolOptions = lib.types.submodule {
    options = {
      version = lib.mkOption {
        type = lib.types.str;
        default = "latest";
        description = "Tool version to install";
      };
      bin = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Override the installed binary name (github backend)";
      };
    };
  };

  toolType = lib.types.attrsOf (lib.types.either lib.types.str toolOptions);

  withPrefix =
    prefix: tools:
    lib.mapAttrs' (name: value: {
      name = "${prefix}:${name}";
      value =
        if lib.isString value then
          value
        else
          lib.filterAttrs (_: v: v != null) { inherit (value) version bin; };
    }) tools;
in
{
  options.programs.mise.tools = {
    aqua = lib.mkOption {
      type = toolType;
      default = { };
      description = "Tools to install via aqua registry backend";
    };

    cargo = lib.mkOption {
      type = toolType;
      default = { };
      description = "Rust packages to install via cargo backend";
    };

    go = lib.mkOption {
      type = toolType;
      default = { };
      description = "Go modules to install via go backend";
    };

    npx = lib.mkOption {
      type = toolType;
      default = { };
      description = "NPM packages to install via npx";
    };

    uvx = lib.mkOption {
      type = toolType;
      default = { };
      description = "Python packages to install via uvx (pipx)";
    };

    github = lib.mkOption {
      type = toolType;
      default = { };
      description = "GitHub releases to install via github backend";
    };

    core = lib.mkOption {
      type = toolType;
      default = { };
      description = "Core mise tools";
    };
  };

  config = lib.mkIf config.programs.mise.enable {
    assertions =
      let
        coreTools = builtins.attrNames config.programs.mise.tools.core;
        invalidTools = builtins.filter (name: lib.hasInfix ":" name) coreTools;
      in
      [
        {
          assertion = invalidTools == [ ];
          message = "mise core tools must not contain ':' backend prefixes. Found: ${builtins.concatStringsSep ", " invalidTools}. Use the appropriate tools section (npx, uvx, github) instead.";
        }
      ];

    programs.mise.globalConfig.tools = lib.mkMerge [
      (withPrefix "aqua" config.programs.mise.tools.aqua)
      (withPrefix "cargo" config.programs.mise.tools.cargo)
      (withPrefix "go" config.programs.mise.tools.go)
      (withPrefix "npm" config.programs.mise.tools.npx)
      (withPrefix "pipx" config.programs.mise.tools.uvx)
      (withPrefix "github" config.programs.mise.tools.github)
      config.programs.mise.tools.core
    ];
  };
}
