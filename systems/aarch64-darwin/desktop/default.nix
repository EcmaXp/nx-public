{
  networking.hostName = "desktop";
  nx = {
    primaryUser = "user";
    roles.desktop.enable = true;
    users.user.enable = true;
  };
}
