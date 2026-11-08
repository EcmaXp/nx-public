{
  inputs,
  lib,
  pkgs,
  ...
}:
let
  kernel = pkgs.fetchurl {
    url = "https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.15/x86_64/vmlinux-6.1.155";
    hash = "sha256-4g5G0MNsVcDRAU6yBXYXGz89kiJg2feSAXrv9Trz1PI=";
  };
  ubuntu = pkgs.fetchurl {
    url = "https://cloud-images.ubuntu.com/releases/noble/release-20260321/ubuntu-24.04-server-cloudimg-amd64-root.tar.xz";
    sha256 = "0xyp9pin8m6ys0gxk6spq83dgdfl3malbvps7v78hrp7sjg3rp0k";
  };
  maskedUnits = [
    "apt-daily-upgrade.timer"
    "apt-daily.timer"
    "snapd.seeded.service"
    "snapd.service"
    "snapd.socket"
    "systemd-networkd-wait-online.service"
    "unattended-upgrades.service"
  ];

  testModule = {
    nixpkgs.hostPlatform = pkgs.stdenv.hostPlatform.system;
    environment.etc = {
      "a/bar" = {
        text = "bar";
        mode = "0700";
        user = "root";
        group = "root";
      };
      "sysctl.d/90-sm-test.conf" = {
        text = "# test";
      };
      "tmpfiles.d/50-replaced.conf" = {
        text = "d /run/replaced-test 0755 root root -";
        replaceExisting = true;
      };
    };
    systemd = {
      tmpfiles.rules = [ "d /run/ephemeral-test 0755 root root -" ];
      services = {
        ephemeral-test = {
          description = "Ephemeral activation test";
          wantedBy = [ "system-manager.target" ];
          serviceConfig = {
            Type = "oneshot";
            RemainAfterExit = true;
          };
          script = "true";
        };
      };
    };
  };

  mkToplevel =
    patches:
    let
      systemManager = import ../../modules/system-manager/patched.nix (
        { inherit inputs pkgs; } // lib.optionalAttrs (patches != null) { inherit patches; }
      );
    in
    systemManager.lib.makeSystemConfig {
      modules = [
        testModule
        ../../modules/system-manager
      ];
      specialArgs = { inherit inputs; };
    };

  testUnit = pkgs.writeText "sm-test.service" ''
    [Unit]
    Description=system-manager ephemeral activation test
    After=multi-user.target
    SuccessAction=reboot-force
    FailureAction=reboot-force

    [Service]
    Type=oneshot
    ExecStart=/usr/local/bin/sm-test
    StandardOutput=tty
    StandardError=tty
    TTYPath=/dev/ttyS0

    [Install]
    WantedBy=multi-user.target
  '';

  nixDaemonUnit = pkgs.writeText "nix-daemon.service" ''
    [Service]
    Type=oneshot
    RemainAfterExit=true
    ExecStart=/bin/true

    [Install]
    WantedBy=multi-user.target
  '';

  mkRootfs =
    toplevel:
    let
      closure = pkgs.closureInfo { rootPaths = [ toplevel ]; };
      testScript = pkgs.replaceVars ./sm-test.sh { inherit toplevel; };
    in
    pkgs.runCommand "system-manager-vm-rootfs"
      {
        nativeBuildInputs = with pkgs; [
          e2fsprogs
          fakeroot
          xz
        ];
      }
      ''
        mkdir -p $out
        fakeroot sh -euc '
          mkdir root
          tar -xJf ${ubuntu} -C root --numeric-owner
          echo "/dev/vda / ext4 defaults 0 1" > root/etc/fstab
          mkdir -p root/etc/nix
          printf original > root/etc/nix/nix.conf
          printf "# original" > root/etc/tmpfiles.d/50-replaced.conf
          touch root/etc/cloud/cloud-init.disabled
          for unit in ${lib.concatStringsSep " " maskedUnits}; do
            ln -sf /dev/null root/etc/systemd/system/$unit
          done
          mkdir -p root/nix/store
          xargs -a ${closure}/store-paths cp -a -t root/nix/store
          install -D -m 755 ${testScript} root/usr/local/bin/sm-test
          install -D -m 644 ${testUnit} root/etc/systemd/system/sm-test.service
          install -D -m 644 ${nixDaemonUnit} root/etc/systemd/system/nix-daemon.service
          mkdir -p root/etc/systemd/system/multi-user.target.wants
          for unit in sm-test.service nix-daemon.service; do
            ln -s ../$unit root/etc/systemd/system/multi-user.target.wants/$unit
          done
          size=$(( $(du -sm root | cut -f1) + 512 ))
          mkfs.ext4 -q -F -L root -d root '"$out"'/rootfs.ext4 "''${size}M"
        '
      '';

  mkRunner =
    {
      name,
      patches,
      expectedFailures ? null,
    }:
    let
      rootfs = mkRootfs (mkToplevel patches);
    in
    pkgs.writeShellApplication {
      inherit name;
      runtimeInputs = with pkgs; [
        coreutils
        firecracker
        gnugrep
      ];
      text = ''
        if [ ! -r /dev/kvm ] || [ ! -w /dev/kvm ]; then
          echo "$0: need rw on /dev/kvm (e.g. sudo setfacl -m u:$USER:rw /dev/kvm)" >&2
          exit 2
        fi

        cache=''${XDG_CACHE_HOME:-$HOME/.cache}
        mkdir -p "$cache"
        work=$(mktemp -d "$cache/${name}.XXXXXX")
        trap 'rm -rf "$work"' EXIT

        cp ${rootfs}/rootfs.ext4 "$work/rootfs.ext4"
        chmod u+w "$work/rootfs.ext4"
        cat > "$work/config.json" << EOF
        {
          "boot-source": {
            "kernel_image_path": "${kernel}",
            "boot_args": "console=ttyS0 reboot=k panic=1 pci=off init=/usr/lib/systemd/systemd"
          },
          "drives": [
            {
              "drive_id": "rootfs",
              "path_on_host": "$work/rootfs.ext4",
              "is_root_device": true,
              "is_read_only": false
            }
          ],
          "machine-config": { "vcpu_count": 2, "mem_size_mib": 1024 }
        }
        EOF

        echo "Booting the test VM (console log: $work/console.log)..."
        status=0
        timeout 600 firecracker --no-api --config-file "$work/config.json" \
          > "$work/console.raw" 2>&1 || status=$?
        tr -d '\r' < "$work/console.raw" > "$work/console.log"
        grep -a '^SMTEST:' "$work/console.log" || true

        keep_log() {
          cp "$work/console.log" "$cache/${name}.last-console.log"
          echo "$0: full log kept at $cache/${name}.last-console.log" >&2
        }

        if grep -aq '^SMTEST: ALL PASSED' "$work/console.log"; then
          result=passed
        elif grep -aq '^SMTEST: FAILED' "$work/console.log"; then
          result=failed
        else
          echo "$0: the VM produced no test summary (firecracker exit $status)" >&2
          tail -n 40 "$work/console.log" >&2
          keep_log
          exit 3
        fi
      ''
      + (
        if expectedFailures == null then
          ''
            [ "$result" = passed ] && exit 0
            keep_log
            exit 1
          ''
        else
          ''
            if [ "$result" = passed ]; then
              echo "$0: expected the bug to reproduce, but every check passed" >&2
              exit 1
            fi
            missing=0
            for desc in ${lib.escapeShellArgs expectedFailures}; do
              if ! grep -aqxF "SMTEST: FAIL $desc" "$work/console.log"; then
                echo "$0: expected failure did not occur: $desc" >&2
                missing=1
              fi
            done
            [ "$missing" -eq 0 ] && exit 0
            keep_log
            exit 1
          ''
      );
      passthru = { inherit kernel rootfs; };
      meta.platforms = [ "x86_64-linux" ];
    };

  patched = mkRunner {
    name = "system-manager-vm-test";
    patches = null;
  };
in
patched.overrideAttrs (old: {
  passthru = old.passthru // {
    unpatched = mkRunner {
      name = "system-manager-vm-test-unpatched";
      patches = [ ];
      expectedFailures = [
        "unit not linked into /etc/systemd/system"
        "sysctl.d entry linked into /run/sysctl.d"
        "systemctl works after healing"
        "ephemeral: installer nix.conf restored"
        "persistent: tmpfiles rule that replaced a file applied"
      ];
    };
  };
})
