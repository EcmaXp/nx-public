{ lib, ... }:
let
  withLatest = tools: lib.genAttrs tools (_: "latest");
in
{
  programs.mise = {
    enable = true;
    tools = {
      npx = withLatest [
        "@toon-format/cli"
        "ccusage"
      ];
      uvx = withLatest [
        "datasette"
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
      ];
      core = withLatest [
        "1password-cli"
        "act"
        "age"
        "age-plugin-yubikey"
        "air"
        "ansible-core"
        "argo"
        "argocd"
        "ast-grep"
        "aws-iam-authenticator"
        "bat"
        "buf"
        "caddy"
        "certstrap"
        "cfssl"
        "cheat"
        "choose"
        "cloudflared"
        "cockroach"
        "curlie"
        "dasel"
        "difftastic"
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
        "helm"
        "helmfile"
        "hexyl"
        "hyperfine"
        "istioctl"
        "jc"
        "jless"
        "jq"
        "just"
        "k6"
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
        "openbao"
        "opentofu"
        "pandoc"
        "peco"
        "pipx"
        "poetry"
        "pre-commit"
        "protobuf"
        "python"
        "ripgrep"
        "ruby"
        "ruff"
        "rust"
        "sd"
        "skaffold"
        "sqlite"
        "steampipe"
        "stern"
        "talosctl"
        "terraform"
        "terragrunt"
        "terramate"
        "tfmigrate"
        "uv"
        "vault"
        "xh"
        "yarn"
        "yq"
        "zellij"
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
