#!/usr/bin/env zsh

_nx_claude_session_prune() {
  _nx_claude_session_prune_session_env
}

_nx_claude_session_prune_session_env() {
  local -a f=( ~/.claude/session-env/*/(setup|sessionstart|cwdchanged|filechanged)-hook-<->.sh(N.m+7) )
  (( $#f )) && command rm -f $f
  local -a d=( ~/.claude/session-env/*(N/m+7^F) )
  (( $#d )) && command rmdir $d 2>/dev/null
  return 0
}

_nx_claude_session_prune
