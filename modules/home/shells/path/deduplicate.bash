__nx_deduplicate_path() {
  local remaining="${PATH-}" seen= entry
  while :; do
    entry="${remaining%%:*}"
    case "$seen:" in
      *":$entry:"*) ;;
      *) seen="$seen:$entry" ;;
    esac
    [[ $remaining == *:* ]] || break
    remaining="${remaining#*:}"
  done
  export PATH="${seen#:}"
}

__nx_deduplicate_path
unset -f __nx_deduplicate_path
