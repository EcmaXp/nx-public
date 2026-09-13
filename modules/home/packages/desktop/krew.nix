{
  config,
  lib,
  inputs,
  pkgs,
  ...
}:
lib.nx.gate config.nx.home.packages.desktop {
  programs.krewfile = {
    enable = true;
    krewPackage = pkgs.krew;
    indexes = {
      default = "https://github.com/kubernetes-sigs/krew-index.git";
    };
    plugins = [
      "ai"
      "kluster-capacity"
      "konfig"
      "krew"
      "minio"
      "neat"
      "node-shell"
      "rook-ceph"
      "sniff"
      "view-secret"
    ];
  };
}
