{
  security.pam.services = {
    sudo_local = {
      touchIdAuth = true;
    };
  };

  # Allow Touch ID for sudo when external monitors are connected
  system.defaults.CustomUserPreferences = {
    "com.apple.security.authorization" = {
      ignoreArd = true;
    };
  };
}
