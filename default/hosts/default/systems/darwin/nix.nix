{
  self,
  lib,
  pkgs,
  ...
}:
{
  system.stateVersion = lib.mkDefault 5; # Did you read the document?
  system.configurationRevision = self.rev or self.dirtyRev or null;

  nix = {
    enable = true;
    extraOptions = (
      if pkgs.stdenv.hostPlatform.system == "aarch64-darwin" then
        "extra-platforms = x86_64-darwin"
      else
        null
    );

    gc = {
      automatic = true;
      interval = [
        {
          Hour = 9;
          Minute = 0;
        }
      ]; # = 9:00 KST
      options = "--delete-older-than 30d";
    };

    optimise = {
      automatic = true;
      interval = [
        {
          Hour = 18;
          Minute = 0;
        }
      ]; # = 18:00 KST
    };
  };
}
