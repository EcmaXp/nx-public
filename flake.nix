{
  description = "EcmaXp's nx";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    nur.url = "github:nix-community/NUR";
    nur.inputs.nixpkgs.follows = "nixpkgs";
    home-manager.url = "github:nix-community/home-manager/release-25.11";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
    nix-darwin.url = "github:LnL7/nix-darwin/nix-darwin-25.11";
    nix-darwin.inputs.nixpkgs.follows = "nixpkgs";
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
    inputs@{
      self,
      nixpkgs,
      nix-darwin,
      home-manager,
      ...
    }:
    let
      # profiles
      default = {
        profiles = [ "default" ];
        username = "user";
      };
      nixos = default // {
        builder = nixpkgs.lib.nixosSystem;
        platform = "nixos";
        system = "x86_64-linux";
      };
      darwin = default // {
        builder = nix-darwin.lib.darwinSystem;
        platform = "darwin";
        system = "aarch64-darwin";
      };
      server = {
        profiles = default.profiles ++ [ "server" ];
      };
      desktop = {
        profiles = default.profiles ++ [ "desktop" ];
      };

      # hosts
      hosts = {
        # servers
        "server" = nixos // server;
        # desktops
        "desktop" = darwin // desktop;
      };

      # functions
      mkSpecialArgs =
        hostname:
        {
          system,
          username,
          platform,
          ...
        }:
        {
          inherit
            inputs
            self
            hostname
            username
            platform
            ;
          pkgs-stable = nixpkgs.legacyPackages.${system};
          pkgs-unstable = inputs.nixpkgs-unstable.legacyPackages.${system};
        };

      buildSystem =
        hostname:
        config@{ builder, profiles, ... }:
        builder {
          specialArgs = mkSpecialArgs hostname config;
          modules = [
            { nixpkgs.hostPlatform = config.system; }
          ]
          ++ map (profile: import ./${profile}) profiles;
        };

      buildHome =
        hostname:
        config@{ system, profiles, ... }:
        home-manager.lib.homeManagerConfiguration {
          pkgs = nixpkgs.legacyPackages.${system};
          extraSpecialArgs = mkSpecialArgs hostname config;
          modules = map (profile: import ./${profile}/users/home.nix) profiles;
        };
      filterHosts = p: nixpkgs.lib.filterAttrs (_: c: c.platform == p) hosts;
    in
    rec {
      nixosConfigurations = nixpkgs.lib.mapAttrs buildSystem (filterHosts "nixos");
      darwinConfigurations = nixpkgs.lib.mapAttrs buildSystem (filterHosts "darwin");
      darwinPackages = nixpkgs.lib.mapAttrs (n: v: v.pkgs) darwinConfigurations;
      homeConfigurations = nixpkgs.lib.mapAttrs' (hostname: config: {
        name = "${config.username}@${hostname}";
        value = buildHome hostname config;
      }) hosts;
    };
}
