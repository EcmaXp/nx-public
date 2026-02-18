{
  environment.variables = {
    HOMEBREW_NO_ANALYTICS = "1";
  };

  homebrew = {
    enable = true;
    onActivation = {
      cleanup = "uninstall";
      autoUpdate = true;
      upgrade = true;
    };
  };
}
