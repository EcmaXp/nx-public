{
  description = "EcmaXp's nx";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    nur.url = "github:nix-community/NUR";
    nur.inputs.nixpkgs.follows = "nixpkgs";
    home-manager.url = "github:nix-community/home-manager/release-26.05";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
    darwin.url = "github:LnL7/nix-darwin/nix-darwin-26.05";
    darwin.inputs.nixpkgs.follows = "nixpkgs";
    snowfall-lib.url = "github:snowfallorg/lib";
    snowfall-lib.inputs.nixpkgs.follows = "nixpkgs";
    nix-index-database.url = "github:nix-community/nix-index-database";
    nix-index-database.inputs.nixpkgs.follows = "nixpkgs";
    ragenix.url = "github:yaxitech/ragenix";
    ragenix.inputs.nixpkgs.follows = "nixpkgs";
    pyproject-nix.url = "github:nix-community/pyproject.nix";
    pyproject-nix.inputs.nixpkgs.follows = "nixpkgs";
    poetry2nix.url = "github:nix-community/poetry2nix";
    poetry2nix.inputs.nixpkgs.follows = "nixpkgs";
    uv2nix.url = "github:adisbladis/uv2nix";
    uv2nix.inputs.pyproject-nix.follows = "pyproject-nix";
    uv2nix.inputs.nixpkgs.follows = "nixpkgs";
    krewfile.url = "github:brumhard/krewfile";
    krewfile.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    inputs:
    inputs.snowfall-lib.mkFlake {
      inherit inputs;
      src = toString ./.;

      snowfall.namespace = "nx";

      channels-config.allowUnfreePredicate =
        pkg:
        builtins.elem (inputs.nixpkgs.lib.getName pkg) [
          "crush"
        ];

      systems.modules.darwin = with inputs; [
        nix-index-database.darwinModules.nix-index
        ragenix.darwinModules.default
      ];

      systems.modules.nixos = with inputs; [
        nix-index-database.nixosModules.nix-index
        ragenix.nixosModules.default
      ];

      homes.modules = with inputs; [
        krewfile.homeManagerModules.krewfile
        nix-index-database.homeModules.nix-index
        ragenix.homeManagerModules.default
      ];
    };
}
