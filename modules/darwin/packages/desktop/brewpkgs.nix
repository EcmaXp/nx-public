{ config, lib, ... }:
lib.nx.gate config.nx.darwin.packages.desktop {
  homebrew = {
    taps = [
      {
        name = "dmtrKovalenko/fff";
        trusted = true;
      }
      {
        name = "macos-fuse-t/cask";
        trusted = true;
      }
      {
        name = "nikitabobko/tap";
        trusted = true;
      }
    ];
    brews = [
      "container"
      "fff-mcp"
      "incus"
      "kubetail"
      "lume"
      "mas"
      "mole"
      "weave"
    ];
    casks = [
      "1password"
      "1password-cli"
      "aerospace"
      "aldente"
      "betterdisplay"
      "claude-code@latest"
      "cloudflare-warp"
      "codex"
      "firefox"
      "font-d2coding"
      "font-jetbrains-mono"
      "font-sf-mono"
      "font-sf-pro"
      "fuse-t"
      "ghostty"
      "google-chrome"
      "google-drive"
      "jordanbaird-ice"
      "karabiner-elements"
      "linearmouse"
      "little-snitch"
      "notion-cli"
      "ollama-app"
      "orbstack"
      "osquery"
      "privileges"
      "raycast"
      "secretive"
      "sf-symbols"
      "slack-cli"
      "tailscale-app"
      "teleport-suite"
      "vibe-island"
      "wireshark-app"
      "zed"
      "zen"
    ];
    userCasks = [
      "balenaetcher"
      "chatgpt"
      "claude"
      "db-browser-for-sqlite"
      "devtoys"
      "fork"
      "github"
      "jetbrains-toolbox"
      "kdiff3"
      "keka"
      "latest"
      "micro-snitch"
      "mole-app"
      "numi"
      "obsidian"
      "ridibooks"
      "slack"
      "typora"
      "visual-studio-code"
      "wooshy"
    ];
  };
}
