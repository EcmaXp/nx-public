{
  programs.fish = {
    interactiveShellInit = ''
      # map cmd-shift-c to copy pwd to clipboard
      bind \e\[99\;10u 'shortcut-copy-pwd'
      bind \e\[12618\;10u 'shortcut-copy-pwd'
      bind Ç 'shortcut-copy-pwd'
      # map cmd-shift-v to paste from clipboard
      bind \e\[118\;10u 'shortcut-paste-pwd'
      bind \e\[12621\;10u 'shortcut-paste-pwd'
      bind ◊ 'shortcut-paste-pwd'
      # map hyper-caps_lock to zi on ghostty
      bind ctrl-R 'shortcut-zi'
      bind ctrl-ㄲ 'shortcut-zi'
      # map hyper-caps_lock to zi on zed
      bind \e\[24\;1~ 'shortcut-zi'
    '';
    functions = {
      "shortcut-copy-pwd" = {
        body = "echo -n (pwd) | pbcopy";
      };
      "shortcut-paste-pwd" = {
        body = ''
          # first line only
          set pwd (string split -n \n (pbpaste))[1]
          if test -d $pwd
            cd $pwd
          else if test -d (path dirname $pwd)
            cd (path dirname $pwd)
          end
          commandline -f repaint
        '';
      };
      "shortcut-zi" = {
        body = "zi ; commandline -f repaint";
      };
    };
  };
}
