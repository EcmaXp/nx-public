{ config, lib, ... }:
lib.nx.gate config.nx.darwin.packages.desktop {
  homebrew = {
    taps = [
      {
        name = "common-fate/granted";
        trusted = true;
      }
      {
        name = "dmtrKovalenko/fff";
        trusted = true;
      }
      {
        name = "nikitabobko/tap";
        trusted = true;
      }
    ];
    brews = [
      "container"
      "docker-credential-helper"
      "fff-mcp"
      "granted"
      "incus"
      "kubetail"
      "lume"
      "mas"
      "mole"
      "sem-cli"
      "sqlcipher" # macOS's python3-sqlcipher uses brew's sqlcipher
      "weave"
      "worktrunk"
    ];
    casks = [
      "1password"
      "1password-cli"
      "aerospace"
      "aldente"
      "betterdisplay"
      "claude-code@latest"
      "cloudflare-warp"
      "codeql"
      "codex"
      "firefox"
      "font-d2coding"
      "font-jetbrains-mono"
      "font-sf-mono"
      "font-sf-pro"
      "gcloud-cli"
      "ghostty"
      "google-chrome"
      "google-drive"
      "jordanbaird-ice"
      "karabiner-elements"
      "linearmouse"
      "little-snitch"
      "notion-cli"
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
      "ollama-app"
      "ridibooks"
      "slack"
      "typora"
      "visual-studio-code"
      "wooshy"
    ];
  };
}
