{ pkgs, ... }:
{
  imports = [
    ./kubectl
  ];

  programs.fish = {
    shellAliases = {
      "rm" = "safe-rm -i";
      "unlink" = "safe-rm -i";
      "ls" = "eza";
      "cat" = "bat";
      "top" = "btop";
      "df" = "duf";
      "du" = "dust";
      "diff" = "difft";
      "hexdump" = "hexyl";
      "edit" = "$EDITOR";
      "age" = "rage";
    };
    shellAbbrs = {
      # just
      "j" = "just";

      # aws
      "a" = "aws";

      # terraform
      "htf" = "terraform";
      "htfi" = "terraform init";
      "htfp" = "terraform plan";
      "htfpo" = "terraform plan -out (basename (pwd))-(date +%Y%m%d).tfplan";
      "htfpp" = "terraform plan -parallelism=256";
      "htfppo" = "terraform plan -parallelism=256 -out (basename (pwd))-(date +%Y%m%d).tfplan";
      "htfa" = "terraform apply";

      # opentofu
      "otf" = "tofu";
      "otfi" = "tofu init";
      "otfp" = "tofu plan";
      "otfpp" = "tofu plan -parallelism=256";
      "otfa" = "tofu apply";

      # dasel
      "dq" = "dasel";

      # git
      "lg" = "lazygit";
      "gab" = "git absorb --base (git-current-origin-branch)";
      "gar" = "git absorb --and-rebase --base (git-current-origin-branch)";
      # https://www.rockyourcode.com/my-git-aliases-and-abbreviations/
      "g" = "git";
      "ga" = "git add";
      "gadn" = "git add . && git diff --cached";
      # interactive git add
      "gai" = "git add -i";
      # git add --patch
      "gap" = "git add -p";
      "gapn" = "git add --intent-to-add . && git add --patch";
      "gbr" = "git branch";
      "gc" = "git commit";
      "gca" = "git commit --amend";
      "gcm" = "git commit -m";
      "gco" = "git checkout";
      "gd" = "git pull";
      # git diff
      "gdf" = "git df";
      "gdif" = "git diff";
      "gdoc" = "git doc";
      "gfom" = "git fetch origin master";
      "gft" = "git fetch";
      "gfu" = "git fetch upstream";
      # git --intent-to-add
      "gin" = "git add -N .";
      "glst" = "git last";
      "gmg" = "git merge";
      "gnew" = "git new";
      "gpu" = "git push";
      "gpl" = "git pull";
      "grb" = "git rebase";
      "grs" = "git reset";
      "gst" = "git status -sb";
      "gsw" = "git switch";

      # git (custom)
      "gsm" = "git switch main || git switch master";

      # igrep
      "igrep" = "ig";

      # python
      "py" = {
        expansion = "python -c '%'";
        setCursor = true;
      };
      "p" = {
        expansion = "pyp '%'";
        setCursor = true;
      };
    }
    // (
      if pkgs.stdenv.isDarwin then
        {
          "pbc" = "pbcopy";
          "pbp" = "pbpaste";
        }
      else
        { }
    );
    functions = {
      "git-current-branch" = {
        body = "git branch --show-current";
      };
      "git-current-origin-branch" = {
        body = "git for-each-ref --format='%(upstream:short)' (git symbolic-ref -q HEAD)";
      };
    };
  };
}
