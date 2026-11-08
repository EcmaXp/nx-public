{ lib, ... }:
{
  gate = opt: cfg: { config = lib.mkIf (opt.enable or false) cfg; };
}
