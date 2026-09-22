_: final: prev: {
  fish = prev.fish.overrideAttrs (_: {
    dontStrip = true;
  });
}
