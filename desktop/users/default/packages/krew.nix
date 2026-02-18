{ inputs, pkgs, ... }:
{
  imports = [
    inputs.krewfile.homeManagerModules.krewfile
  ];

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
