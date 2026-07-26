# nx

Unified Nix configuration for macOS (nix-darwin) and NixOS with standalone Home Manager, built on [snowfall-lib](https://github.com/snowfallorg/lib).

Host entrypoints live in `systems/`, Home Manager entrypoints in `homes/`, and everything they enable in `modules/`. The `nx` CLI and supporting tooling live in `tools/nx/`.

## How It Works

Snowfall auto-imports every nested `default.nix` under `modules/{darwin,nixos,home}/`; whether a module applies is decided by `nx.*` options, not imports. A host entrypoint is just a few flags:

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

Role and user hubs fan these flags out to fine-grained `nx.<class>.*` enables, and each leaf module gates itself with `lib.mkIf`. Most managed dotfiles are out-of-store symlinks into the live checkout at `~/nx`, so edits apply without rebuilding.

## Usage

- `nx os build|switch|test`: system configuration (via `nh`)
- `nx home build|switch`: Home Manager configuration (via `nh`)
- `nx sync`: sync YAML configs into managed files
- `nx lock flake|mise|uv`: update lockfiles with dependency cooldowns
- `nx refresh`: full refresh (sync, switch system and home, upgrade tools, clean)

Everything is formatted with `treefmt`.
