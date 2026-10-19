#!/usr/bin/env zsh

_nx_claude_session_env_dedup() {
  local sid=$(command jq -r .session_id 2>/dev/null)
  [[ $sid == *[^0-9a-f-]* ]] && sid=
  [[ -n $sid ]] || return 0
  local f
  for f in ~/.claude/session-env/$sid/sessionstart-hook-<->.sh(N.); do
    _nx_claude_session_env_dedup_file $f
  done
  return 0
}

_nx_claude_session_env_dedup_file() {
  setopt local_options extended_glob
  local -a lines=( ${(f)"$(<$1)"} )
  (( $#lines )) || return 0
  (( ${#lines:#export [A-Za-z_][A-Za-z0-9_]#=*} == 0 )) || return 0
  local tmp=$1.dedup.$$
  command awk 'NF { line[NR] = $0; last[$0] = NR } END { for (i = 1; i <= NR; i++) if (i in line && last[line[i]] == i) print line[i] }' $1 >$tmp || { command rm -f $tmp; return 0 }
  if command cmp -s $1 $tmp; then
    command rm -f $tmp
  else
    command mv -f $tmp $1
  fi
}

_nx_claude_session_env_dedup
