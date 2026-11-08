{ pkgs, lib, ... }:
let
  py = pkgs.python3Packages;
  pyobjc-framework-ScriptingBridge = py.pyobjc-framework-WebKit.overridePythonAttrs (old: {
    pname = "pyobjc-framework-ScriptingBridge";
    sourceRoot = "${old.src.name}/pyobjc-framework-ScriptingBridge";
    pythonImportsCheck = [ "ScriptingBridge" ];
  });
in
py.buildPythonApplication rec {
  pname = "sidepulse";
  version = "1.20260902";
  pyproject = true;

  src = pkgs.fetchFromGitHub {
    owner = "inteliwear";
    repo = "sidepulse";
    rev = "f9f1d845153b4f3bc23a5e11d87ecde2e36d6479";
    hash = "sha256-vLdVsOYuXfIuME6WxNGtZ5veX2tu3A9XaijXrrg85zQ=";
  };

  build-system = with py; [
    setuptools
    setuptools-scm
  ];

  env.SETUPTOOLS_SCM_PRETEND_VERSION_FOR_SIDEPULSE = version;

  dependencies = with py; [
    pyobjc-framework-Cocoa
    pyobjc-framework-Quartz
    pyobjc-framework-WebKit
    pyobjc-framework-ScriptingBridge
  ];

  postPatch = ''
    substituteInPlace src/sidepulse/status_bar_launch.py \
      --replace-fail 'command = [executable, "-m", "sidepulse", "status-bar", "--foreground"]' \
                     'command = ["${placeholder "out"}/bin/sidepulse", "status-bar", "--foreground"]'
  '';

  doCheck = false;
  pythonImportsCheck = [
    "sidepulse"
    "sidepulse.cli"
  ];

  meta = with lib; {
    description = "CLI and menu-bar app that drives SidePulse LED devices from AI agent hook state";
    homepage = "https://sidepulse.io";
    license = licenses.mit;
    platforms = platforms.darwin;
    mainProgram = "sidepulse";
  };
}
