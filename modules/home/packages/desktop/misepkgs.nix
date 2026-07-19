{ config, lib, ... }:
let
  withLatest = tools: lib.genAttrs tools (_: "latest");
in
lib.nx.gate config.nx.home.packages.desktop {
  programs.mise = {
    enable = true;
    tools = {
      npx = withLatest [
        "@toon-format/cli"
        "ccusage"
        "next"
      ];
      uvx = withLatest [
        "claude-swap"
        "datasette"
        "headroom-ai[all]"
        "httpie"
        "llm"
        "mypy"
        "pyp"
        "pyright"
        "pz"
        "reloader.py"
        "virtualenv"
        "visidata"
      ];
      github = withLatest [
        "ByteNess/aws-vault"
        "cilium/cilium-cli"
        "datadog-labs/pup"
        "googleworkspace/cli"
        "grafana/k6"
        "helmfile/helmfile"
        "jgm/pandoc"
        "lingrino/vaku"
        "ogulcancelik/herdr"
        "openbao/openbao"
        "sharkdp/bat"
        "siderolabs/talos"
        "str4d/age-plugin-yubikey"
        "theryangeary/choose"
        "wezm/git-grab"
        "zellij-org/zellij"
      ];
      core = {
        helm = "4";
      }
      // withLatest [
        "act"
        "age"
        "air"
        "ansible-core"
        "argo"
        "argocd"
        "ast-grep"
        "aws-iam-authenticator"
        "azure-cli"
        "buf"
        "bun"
        "caddy"
        "cfssl"
        "cheat"
        "cloudflared"
        "cockroach"
        "curlie"
        "dasel"
        "dotnet"
        "duckdb"
        "dust"
        "dyff"
        "eksctl"
        "fd"
        "ffmpeg"
        "fx"
        "gh"
        "go"
        "golangci-lint"
        "grpcurl"
        "gum"
        "hexyl"
        "hyperfine"
        "istioctl"
        "jless"
        "jq"
        "just"
        "k9s"
        "kn"
        "krew"
        "kube-linter"
        "kubeconform"
        "kubectl"
        "kubectx"
        "kubeshark"
        "kubie"
        "kustomize"
        "lazydocker"
        "lazygit"
        "node"
        "opentofu"
        "peco"
        "pipx"
        "pre-commit"
        "protobuf"
        "python"
        "ripgrep"
        "rtk"
        "ruby"
        "ruff"
        "rust"
        "sd"
        "skaffold"
        "sqlite"
        "steampipe"
        "stern"
        "terraform"
        "terragrunt"
        "terramate"
        "tfmigrate"
        "uv"
        "vault"
        "xh"
        "yarn"
        "yq"
      ];
    };
    # for reshim
    enableFishIntegration = false;
  };

  programs = {
    fish.interactiveShellInit = ''
      mise activate fish --shims | source
    '';
  };
}
