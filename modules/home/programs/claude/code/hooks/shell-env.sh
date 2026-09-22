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

gw() {
  if [[ $1 == -u ]]; then unset GIT_WORK_TREE GIT_DIR; return 0; fi
  local top cwd_top
  top=$(unset GIT_DIR GIT_WORK_TREE; git -C ${1:-.} rev-parse --show-toplevel) || return
  if [[ -n $GIT_WORK_TREE$GIT_DIR && ( $GIT_WORK_TREE != "$top" || $GIT_DIR != "$top/.git" ) ]]; then
    print -ru2 -- "gw: already pinned to ${GIT_WORK_TREE:-$GIT_DIR}; run gw -u first"
    return 1
  fi
  # Pinning the repo that already encloses the cwd changes nothing, but it reads as if
  # paths were root-relative when git still resolves them against the cwd.
  cwd_top=$(unset GIT_DIR GIT_WORK_TREE; git rev-parse --show-toplevel 2>/dev/null)
  if [[ $cwd_top == $top && $PWD != $top ]]; then
    print -ru2 -- "gw: cwd is already inside $top; drop the gw call, or cd to the top."
    print -ru2 -- "gw: pinning here would leave relative pathspecs resolving against $PWD."
    return 1
  fi
  export GIT_WORK_TREE=$top GIT_DIR=$top/.git
  print -r -- "gw: GIT_WORK_TREE=$top"
}

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
