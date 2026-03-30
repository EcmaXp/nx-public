{
  home.symlink = {
    ".config/zed/snippets" = ./snippets;
    ".config/zed/themes" = ./themes;
    ".config/zed/settings.json" = ./settings.json5;
    ".config/zed/keymap.json" = ./keymap.json5;
    ".config/zed/tasks.json" = ./tasks.json5;
  };

  programs.fish = {
    interactiveShellInit = ''
      if test "$ZED_TERM" = "true" -a "$TERM" = "alacritty" -a -z "$__ZED_LANG_PATCHED"
        set -x __ZED_LANG_PATCHED 1
        if test "$LANG" = "" -a "$LC_ALL" = "en_US.UTF-8" -a "$LC_CTYPE" = "en_US.UTF-8"
          set -gx LANG "ko_KR.UTF-8"
          set -e LC_ALL LC_CTYPE
        end
      end
    '';
  };
}
