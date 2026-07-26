#!/usr/bin/env zsh

# Bash-tool commands run through `eval`; strip the model's stray \! escapes
# (non-interactive zsh has no history expansion) while keeping \\! expressible.
if [[ -n $CLAUDECODE ]]; then
  function eval {
    local b=$'\x5c' x=$'\x21' s=$'\x01' # backslash, bang, sentinel
    local -a a=( "${@//$b$b$x/$s}" )  # protect \\!
    a=( "${a[@]//$b$x/$x}" )          # strip stray \!
    builtin eval "${a[@]//$s/$b$b$x}" # restore \\!
  }
fi

_nx_claude_shell_env() {
  _nx_claude_scratchpad_env
  _nx_claude_op_cache_session_key_env
}

_nx_claude_scratchpad_env() {
  [[ -n $SCRATCHPAD ]] && return
  local -a sp=( ${CLAUDE_TMPDIR:-${CLAUDE_CODE_TMPDIR:-/tmp/claude-$UID}}/*/$CLAUDE_CODE_SESSION_ID/scratchpad(N/) )
  (( $#sp )) || return 0
  export SCRATCHPAD=$sp[1]
}

_nx_claude_op_cache_session_key_env() {
  [[ -n $OP_CACHE_SESSION_KEY ]] && return
  local keyfile=${CLAUDE_TMPDIR:-${CLAUDE_CODE_TMPDIR:-/tmp/claude-$UID}}/op-cache-key-$CLAUDE_CODE_SESSION_ID
  [[ -s $keyfile ]] || return 0
  export OP_CACHE_SESSION_KEY=$(<$keyfile)
}

if [[ -n $CLAUDE_CODE_SESSION_ID ]]; then
  _nx_claude_shell_env
fi
