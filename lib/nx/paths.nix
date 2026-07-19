{ lib, ... }:
{
  mkRelativePath =
    self: homeDirectory: source:
    "${homeDirectory}/nx${lib.removePrefix (toString self) (toString source)}";
}
