{
  imports = [
    ./aliases/base.nix
  ];

  home.sessionVariables = {
    EDITOR = "micro";
  };

  programs.fish = {
    enable = true;
    interactiveShellInit = ''
      set -g fish_greeting
    '';
  };

  programs.nushell = {
    envFile.text = ''
      $env.config = {
        show_banner: false,
      }
    '';
  };
}
