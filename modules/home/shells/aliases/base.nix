{
  programs.fish = {
    shellAbbrs = {
      # aws
      "a" = "aws";

      # terraform
      "htf" = "terraform";

      # opentofu
      "otf" = "tofu";
      "opentofu" = "tofu";

      # git
      # https://www.rockyourcode.com/my-git-aliases-and-abbreviations/
      "g" = "git";
      "gft" = "git fetch";
      "gpu" = "git push";
      "gpl" = "git pull";
      "gst" = "git status -sb";
      "gsw" = "git switch";

      # git (custom)
      "gsm" = "git switch main || git switch master";
    };
  };
}
