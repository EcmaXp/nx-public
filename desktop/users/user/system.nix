{
  platform,
  config,
  lib,
  ...
}:
{ }
// (
  if platform == "darwin" then
    {
      system.defaults = {
        loginwindow = {
          LoginwindowText = lib.strings.concatLines [
            config.networking.hostName
            "user@example.com"
          ];
        };
      };
    }
  else
    { }
)
