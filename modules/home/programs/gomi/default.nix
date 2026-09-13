{ config, lib, ... }:
lib.nx.gate config.nx.home.programs.gomi {
  home.file = {
    # Restated defaults, not preferences: gomi parses this onto a zero Config.
    ".config/gomi/config.yaml".text = ''
      core:
        trash:
          strategy: xdg
          home_fallback: true
          forbidden_paths:
            - "$HOME/.local/share/Trash"
            - "$HOME/.trash"
            - "$XDG_DATA_HOME/Trash"
            - /tmp/Trash
            - /var/tmp/Trash
            - "$HOME/.gomi"
            - /
            - /etc
            - /usr
            - /var
            - /bin
            - /sbin
            - /lib
            - /lib64
        restore:
          confirm: true
          verbose: true
        permanent_delete:
          enable: false

      ui:
        density: spacious
        paginator_type: dots
        preview:
          syntax_highlight: true
          colorscheme: nord
          directory_command: ls -GF -1 -A --color=always
        style:
          list_view:
            indent_on_select: true

      history:
        include:
          within_days: 365
        exclude:
          files:
            - .DS_Store
          size:
            min: 0KB
            max: 10GB

      logging:
        enabled: false
    '';

    # The sandbox allowlists only this leaf, so gomi cannot create it itself.
    ".local/share/Trash/.keep".text = "";
  };
}
