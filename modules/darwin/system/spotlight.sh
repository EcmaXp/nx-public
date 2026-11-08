volume=${NX_SPOTLIGHT_VOLUME:-/System/Volumes/Data}
apps=${NX_SPOTLIGHT_APPS:-/Applications}
plist=$volume/.Spotlight-V100/VolumeConfiguration.plist
want=$(jq -r '.[]' <<<"$wantJson" | LC_ALL=C sort -u)
flipped=0

warn() { printf '\e[1;33mspotlight: %s\e[0m\n' "$*" >&2; }

indexing() {
  local out
  out=$(timeout 15 mdutil -s "$1" 2>&1) || {
    echo unknown
    return
  }
  case $out in
  *"Indexing enabled"*) echo enabled ;;
  *"Indexing disabled"*) echo disabled ;;
  *) echo unknown ;;
  esac
}

current() {
  [[ -r $plist ]] && plutil -lint -s "$plist" >/dev/null || return 2
  local j
  if j=$(plutil -extract Exclusions json -o - "$plist" 2>/dev/null); then
    jq -r '.[]' <<<"$j" | LC_ALL=C sort -u
  fi
}

mas_ids() {
  {
    PATH="$masPath:$PATH" timeout 20 sudo --preserve-env=PATH --user="$brewUser" \
      --set-home env mas list 2>/dev/null || true
  } | awk '$1 ~ /^[0-9]+$/ {print $1}'
}

state=$(indexing "$volume")
if [[ $state == unknown ]]; then
  warn "indexing state of $volume is unknown (sandboxed or mds down)"
  exit 1
fi

if [[ ! -e $plist && $state == disabled ]]; then
  timeout 30 mdutil -i off "$volume" >/dev/null || true
fi
for _ in {1..15}; do
  [[ -e $plist ]] && break
  sleep 1
done
if ! have=$(current); then
  warn "$plist missing or unreadable; leaving indexing unchanged"
  exit 1
fi

if [[ $have != "$want" ]]; then
  warn "exclusions: removing [$(comm -23 <(echo "$have") <(echo "$want") | xargs)]" \
    "adding [$(comm -13 <(echo "$have") <(echo "$want") | xargs)]"
  tmp=$(mktemp "$plist.XXXXXX")
  trap 'rm -f "$tmp"' EXIT
  plutil -replace Exclusions -json "$wantJson" -o "$tmp" "$plist"
  plutil -lint -s "$tmp" >/dev/null
  chown --reference="$plist" "$tmp"
  chmod --reference="$plist" "$tmp"
  mv -f "$tmp" "$plist"
  killall mds || true
  for _ in {1..15}; do
    [[ $(indexing "$volume") != unknown ]] && break
    sleep 1
  done
  sleep 3
  if [[ $(current || true) != "$want" ]]; then
    warn "mds did not keep the exclusions; not enabling"
    exit 1
  fi
fi

if [[ $(indexing "$volume") == disabled ]]; then
  timeout 30 mdutil -i on "$volume" >/dev/null || true
  if [[ $(indexing "$volume") != enabled ]]; then
    warn "could not enable indexing on $volume"
    exit 1
  fi
  flipped=1
fi

[[ -n $declaredIds ]] || exit 0

expected=$({ find "$apps" -maxdepth 4 -type f -path '*/Contents/_MASReceipt/receipt' 2>/dev/null || true; } | wc -l)
SECONDS=0
while :; do
  got=$(mas_ids | wc -l)
  ((got >= expected || SECONDS >= 60)) && break
  sleep 2
done

if ((flipped)); then
  warn "indexing was just enabled; skipping brew bundle for this run"
  warn "next switch uninstalls: [$(comm -23 <(mas_ids | LC_ALL=C sort) <(LC_ALL=C sort <<<"$declaredIds") | xargs)]"
  exit 1
fi
if ((got < expected)); then
  warn "mas list has $got of $expected App Store apps"
  exit 1
fi
