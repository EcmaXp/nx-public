{
  config,
  lib,
  pkgs,
  ...
}:
let
  historyHooks =
    pkgs.runCommand "rex-history.fish" { nativeBuildInputs = [ config.programs.fish.package ]; }
      ''
        bin=$(mktemp -d)
        cat > $bin/nx <<'EOF'
        #!${pkgs.runtimeShell}
        echo "$* DIRENV_DIR=''${DIRENV_DIR-}" >> "$HOME/calls"
        EOF
        chmod +x $bin/nx
        shell() {
          last=$1
          shift
          home=$(mktemp -d)
          mkdir -p $home/.cache/nx/agent/rex-blocks
          echo '{}' > $home/.cache/nx/agent/rex-blocks/b1.json
          touch $home/calls
          stamp=$home/.local/state/nx/rex-history/b1.returned
          env HOME=$home PATH=$bin:$PATH REX_BLOCK=block:b1 "$@" fish --no-config -l -i -c "
            function __direnv_export_eval --on-event fish_prompt
                set -gx DIRENV_DIR -/project
            end
            source ${./history.fish}
            emit fish_prompt
            emit fish_prompt
            $last
            emit fish_postexec make
            ''${after-true}
          " </dev/null || true
        }
        called() {
          for _ in $(seq 100); do
            grep -q "$1" $home/calls && return 0
            sleep 0.1
          done
          return 1
        }
        a_restored_login_shell_restores_once_after_direnv_loads() {
          shell true
          test "$(grep -c 'restore-block block:b1 DIRENV_DIR=-/project$' $home/calls)" = 1
        }
        a_shell_started_inside_the_block_never_restores() {
          shell true NX_REX_BLOCK_SHELL=block:b1
          test "$(grep -c restore-block $home/calls)" = 0
        }
        a_finished_command_is_saved_and_marks_the_shell_returned() {
          shell true NX_REX_BLOCK_SHELL=block:b1
          test -e $stamp && called save-block
        }
        a_hung_up_command_is_neither_saved_nor_marked_returned() {
          shell "sh -c 'exit 129'" NX_REX_BLOCK_SHELL=block:b1
          sleep 1
          test ! -e $stamp && test "$(grep -c save-block $home/calls)" = 0
        }
        a_stopped_job_does_not_mark_the_shell_returned() {
          after='kill -CONT $stopped; kill $stopped' shell 'sleep 30 &; set -g stopped $last_pid; kill -STOP $stopped; for i in (seq 50); LC_ALL=C jobs | string match -qr "\tstopped\t"; and break; sleep 0.1; end' NX_REX_BLOCK_SHELL=block:b1
          test ! -e $stamp
        }
        a_restored_login_shell_restores_once_after_direnv_loads
        a_shell_started_inside_the_block_never_restores
        a_finished_command_is_saved_and_marks_the_shell_returned
        a_hung_up_command_is_neither_saved_nor_marked_returned
        a_stopped_job_does_not_mark_the_shell_returned
        cp ${./history.fish} $out
      '';
in
lib.nx.gate config.nx.home.programs.rex {
  home.symlink = {
    ".config/rex/init.lua" = ./init.lua;
  };

  programs = {
    fish = {
      interactiveShellInit = lib.mkAfter ''
        source ${historyHooks}
      '';
    };
  };
}
