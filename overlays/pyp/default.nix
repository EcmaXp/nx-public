_: final: prev: {
  pyp = (prev.pyp.override { python3Packages = final.python314Packages; }).overridePythonAttrs (old: {
    disabledTests = (old.disabledTests or [ ]) ++ [ "test_weird_assignments" ];
  });
}
