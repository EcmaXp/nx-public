_: final: prev: {
  safe-rm = prev.safe-rm.overrideAttrs (oldAttrs: {
    postPatch = ''
      substituteInPlace src/main.rs \
        --replace "/bin/rm" "/run/current-system/sw/bin/rm"
    '';
  });
}
