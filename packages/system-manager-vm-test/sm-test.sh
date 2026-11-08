#!/bin/bash
set -u

toplevel=@toplevel@
failures=0

section() { echo "SMTEST: --- $*"; }

check() {
  local desc=$1
  shift
  if "$@" > /dev/null 2>&1; then
    echo "SMTEST: ok   $desc"
  else
    echo "SMTEST: FAIL $desc"
    failures=$((failures + 1))
  fi
}

absent() { ! test -e "$1" && ! test -L "$1"; }
real_dir() { test -d "$1" && ! test -L "$1"; }
file_is() { test -f "$1" && ! test -L "$1" && [ "$(cat "$1")" = "$2" ]; }
unit_active() { systemctl is-active --quiet ephemeral-test.service; }
systemctl_works() { systemctl list-units --no-pager; }
daemon_invocation() { systemctl show -p InvocationID --value nix-daemon.service; }

nix_conf=/etc/nix/nix.conf
nix_conf_backup=/etc/nix/nix.conf.system-manager-backup

activate() {
  local log rc
  log=$(timeout 300 "$toplevel/bin/activate" "$@" 2>&1)
  rc=$?
  sed 's/^/  | /' <<< "$log" > /dev/console
  [ "$rc" -eq 0 ] && ! grep -q ERROR <<< "$log"
}

add_sentinel() {
  mkdir -p /run/systemd/system
  printf '[Unit]\nDescription=sentinel\n' > /run/systemd/system/sentinel.service
}

section "1. ephemeral activation keeps existing runtime units"
add_sentinel
check "first ephemeral activation succeeds without ERROR" activate --ephemeral
check "/run/systemd/system stays a real directory" real_dir /run/systemd/system
check "existing runtime unit survives ephemeral activation" test -f /run/systemd/system/sentinel.service
check "systemctl works after ephemeral activation" systemctl_works
check "unit active after ephemeral activation" unit_active
check "installer nix.conf untouched by ephemeral activation" file_is $nix_conf original

section "2. ephemeral entries land under /run only"
check "/run/etc/a/bar written" grep -qx bar /run/etc/a/bar
check "/etc/a/bar not written" absent /etc/a/bar
check "unit linked into /run/systemd/system" test -L /run/systemd/system/ephemeral-test.service
check "unit not linked into /etc/systemd/system" absent /etc/systemd/system/ephemeral-test.service
check "tmpfiles rules linked into /run/tmpfiles.d" test -L /run/tmpfiles.d/00-system-manager.conf
check "tmpfiles rules not linked into /etc/tmpfiles.d" absent /etc/tmpfiles.d/00-system-manager.conf
check "tmpfiles rule applied" real_dir /run/ephemeral-test
check "sysctl.d entry linked into /run/sysctl.d" test -L /run/sysctl.d/90-sm-test.conf

section "3. a runtime unit dir symlink left by older versions is healed"
rm -rf /run/systemd/system /run/etc/systemd
ln -s /run/etc/systemd/system /run/systemd/system
check "ephemeral activation over a leftover symlink succeeds without ERROR" activate --ephemeral
check "leftover /run/systemd/system symlink healed" real_dir /run/systemd/system
check "systemctl works after healing" systemctl_works
check "unit active after healing" unit_active

section "4. deactivating an ephemeral activation removes its entries only"
add_sentinel
check "ephemeral deactivate succeeds" "$toplevel/bin/deactivate"
check "/run/systemd/system stays a real directory after deactivate" real_dir /run/systemd/system
check "existing runtime unit survives deactivate" test -f /run/systemd/system/sentinel.service
check "systemctl works after ephemeral deactivate" systemctl_works
check "/run/etc/a/bar removed by deactivate" absent /run/etc/a/bar
check "unit link removed by deactivate" absent /run/systemd/system/ephemeral-test.service
check "tmpfiles rules link removed by deactivate" absent /run/tmpfiles.d/00-system-manager.conf
check "installer nix.conf untouched by ephemeral deactivate" file_is $nix_conf original

section "5. persistent and ephemeral activations replace each other"
rm -rf /run/replaced-test
check "persistent activation succeeds" activate
check "persistent: /etc/a/bar written" grep -qx bar /etc/a/bar
check "persistent: /run/etc/a/bar absent" absent /run/etc/a/bar
check "persistent: unit active" unit_active
check "persistent: nix.conf is a managed link" test -L $nix_conf
check "persistent: installer nix.conf backed up" file_is $nix_conf_backup original
check "persistent: tmpfiles rule that replaced a file applied" real_dir /run/replaced-test
check "persistent: sysctl.d entry in /etc/sysctl.d" test -L /etc/sysctl.d/90-sm-test.conf
daemon=$(daemon_invocation)

check "ephemeral activation after persistent succeeds" activate --ephemeral
check "ephemeral: /etc/a/bar removed" absent /etc/a/bar
check "ephemeral: unit no longer in /etc/systemd/system" absent /etc/systemd/system/ephemeral-test.service
check "ephemeral: /run/etc/a/bar written" grep -qx bar /run/etc/a/bar
check "ephemeral: unit active" unit_active
check "ephemeral: systemctl works" systemctl_works
check "ephemeral: installer nix.conf restored" file_is $nix_conf original
check "ephemeral: nix.conf backup consumed" absent $nix_conf_backup
check "nix-daemon restarted for the installer nix.conf" test "$(daemon_invocation)" != "$daemon"
daemon=$(daemon_invocation)

check "persistent activation after ephemeral succeeds" activate
check "persistent again: /etc/a/bar written" grep -qx bar /etc/a/bar
check "persistent again: /run/etc/a/bar removed" absent /run/etc/a/bar
check "persistent again: unit link removed from /run/systemd/system" absent /run/systemd/system/ephemeral-test.service
check "persistent again: unit active" unit_active
check "persistent again: systemctl works" systemctl_works
check "persistent again: nix.conf is a managed link" test -L $nix_conf
check "persistent again: backup still holds the installer nix.conf" file_is $nix_conf_backup original
check "persistent again: nix-daemon restarted for the managed nix.conf" test "$(daemon_invocation)" != "$daemon"
daemon=$(daemon_invocation)
check "repeated persistent activation succeeds" activate
check "unchanged nix.conf does not restart nix-daemon" test "$(daemon_invocation)" = "$daemon"

section "6. deactivating a persistent activation restores the installer files"
check "persistent deactivate succeeds" "$toplevel/bin/deactivate"
check "deactivate restores the installer nix.conf" file_is $nix_conf original
check "deactivate consumes the nix.conf backup" absent $nix_conf_backup
check "deactivate restores the replaced tmpfiles.d file" file_is /etc/tmpfiles.d/50-replaced.conf "# original"
check "systemctl works after persistent deactivate" systemctl_works

if [ "$failures" -eq 0 ]; then
  echo "SMTEST: ALL PASSED"
else
  echo "SMTEST: FAILED ($failures checks)"
fi
