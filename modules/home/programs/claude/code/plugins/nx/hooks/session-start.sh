#!/usr/bin/env zsh

_nx_claude_session_start() {
  local sid=$(command jq -r .session_id 2>/dev/null)
  [[ $sid == *[^0-9a-f-]* ]] && sid=
  _nx_claude_session_start_tmpdir
  _nx_claude_session_start_op_cache_session_key "$sid"
}

_nx_claude_session_start_tmpdir() {
  local dir
  for dir in \
    /tmp/claude \
    /tmp/claude-$UID \
    ${CLAUDE_TMPDIR:-${CLAUDE_CODE_TMPDIR:-/tmp/claude-$UID}}
  do
    [[ -d $dir ]] || command mkdir $dir 2>/dev/null
  done
}

_nx_claude_session_start_op_cache_session_key() {
  [[ -n $1 ]] || return 0
  local keyfile=${CLAUDE_TMPDIR:-${CLAUDE_CODE_TMPDIR:-/tmp/claude-$UID}}/op-cache-key-$1
  [[ -s $keyfile ]] && return
  ( umask 077; command openssl rand -hex 32 >$keyfile ) 2>/dev/null
}

_nx_claude_session_start
