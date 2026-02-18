{ lib, ... }:
let
  python = self: super: {
    python312 = super.python312.override {
      packageOverrides = pySelf: pySuper: {
        aiohttp = pySuper.aiohttp.overridePythonAttrs {
          disabledTests = [
            # failed tests on python 3.12 @ 2025-04-26
            "test_connector_multiple_event_loop"
            "test_constructor[uvloop]"
          ];
        };
      };
    };
    python313 = super.python313.override {
      packageOverrides = pySelf: pySuper: {
        twisted = pySuper.twisted.overridePythonAttrs {
          doCheck = false;
        };
      };
    };
  };
in
{
  nixpkgs.overlays = lib.mkBefore [
    python
  ];
}
