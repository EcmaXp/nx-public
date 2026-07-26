{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.ctx {
  programs.mise.tools.github = {
    # pinned: `latest` resolves to the asset-less v0.19.0 under the 7d cooldown
    "ctxrs/ctx" = {
      version = "0.24.0";
      bin = "ctx";
    };
  };
}
