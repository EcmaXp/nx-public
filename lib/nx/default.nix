{ lib, ... }@args:
let
  imports = [
    ./brew.nix
    ./gate.nix
    ./paths.nix
  ];
in
lib.foldl' lib.recursiveUpdate { } (map (path: import path args) imports)
