# nx

One Nix flake for macOS (nix-darwin) and NixOS machines:
system configuration, standalone Home Manager homes, custom packages, and the `nx` CLI that operates them.
Built on [snowfall-lib](https://github.com/snowfallorg/lib).

Host entrypoints live in `systems/`, Home Manager entrypoints in `homes/`, and everything they enable in `modules/`.
Custom packages and overlays live in `packages/` and `overlays/`, and the `nx` CLI with its supporting tooling in `tools/`.

## How it works

Snowfall auto-imports every nested `default.nix` under `modules/{darwin,nixos,home}/`;
whether a module applies is decided by `nx.*` options, not imports.
A host entrypoint is just a few flags:

```nix
{
  networking.hostName = "desktop";

  nx = {
    primaryUser = "user";
    roles.desktop.enable = true;
    users.user.enable = true;
  };
}
```

Role and user hubs fan these flags out to fine-grained `nx.<class>.*` enables, and each leaf module gates itself with `lib.nx.gate`.
Most managed dotfiles are out-of-store symlinks into the live checkout at `~/nx`, so edits apply without rebuilding.

## Usage

- `nx refresh`: sync configs, switch system and home, then upgrade Homebrew and mise tools
- `nx lock all`: update every lockfile with dependency cooldowns
- `nx clean`: garbage-collect the Nix store and tool caches

Everything is formatted with `treefmt`.
