#!/usr/bin/env zsh

_nx_claude_session_end() {
  local sid=$(command jq -r .session_id 2>/dev/null)
  [[ $sid == *[^0-9a-f-]* ]] && sid=
  _nx_claude_session_end_session_env "$sid"
  _nx_claude_session_end_op_cache_session_key "$sid"
  _nx_claude_session_end_op_inject_cache
}

_nx_claude_session_end_session_env() {
  [[ -n $1 ]] || return 0
  command rm -rf ~/.claude/session-env/$1
}

_nx_claude_session_end_op_cache_session_key() {
  [[ -n $1 ]] || return 0
  command rm -f ${CLAUDE_TMPDIR:-${CLAUDE_CODE_TMPDIR:-/tmp/claude-$UID}}/op-cache-key-$1
}

_nx_claude_session_end_op_inject_cache() {
  local -a f=( ${CLAUDE_TMPDIR:-${CLAUDE_CODE_TMPDIR:-/tmp/claude-$UID}}/op-inject-*.enc(N) )
  (( $#f )) || return 0
  command rm -f $f
}

_nx_claude_session_end
