#!/usr/bin/env bash
# Dump everything a port needs to read out of the phone.
# The phone is in the running system with USB debugging on and authorised.
#
# Run from the project root:  ADB=tools/platform-tools/adb.exe bash device/collect.sh
set -u

# Git Bash rewrites /vendor/... into a Windows path before adb ever sees it.
export MSYS_NO_PATHCONV=1
export MSYS2_ARG_CONV_EXCL='*'

OUT="$(cd "$(dirname "$0")" && pwd)"
ADB="${ADB:-adb}"

die() { echo "!! $*" >&2; exit 1; }

command -v "$ADB" >/dev/null 2>&1 || [ -x "$ADB" ] || die "adb not found: $ADB"
"$ADB" get-state >/dev/null 2>&1 || die "no device: check the cable and the RSA prompt"

echo "device: $("$ADB" shell getprop ro.product.device | tr -d '\r')"
echo "build : $("$ADB" shell getprop ro.build.version.incremental | tr -d '\r')"
echo

sh() {
    local name="$1"; shift
    "$ADB" shell "$@" 2>/dev/null | tr -d '\r' > "$OUT/$name" || echo "  (failed) $name"
}

pull() {
    # Flatten the remote path so two pulls cannot collide on a basename.
    # The remote side must stay a raw POSIX path; the local side must be a
    # Windows path, because MSYS_NO_PATHCONV stops both conversions at once.
    local src="$1"
    local flat="${src#/}"
    local posix="$OUT/${flat//\//__}"
    local win
    win="$(cygpath -w "$posix")"
    rm -rf "$posix"
    if "$ADB" pull "$src" "$win" >/dev/null 2>&1; then
        echo "  pulled $src"
    else
        echo "  (failed) $src"
    fi
}

echo "== plain text =="
sh cmdline.txt      cat /proc/cmdline
sh partitions.txt   cat /proc/partitions
sh mounts.txt       mount
sh selinux.txt      getenforce
sh lsmod.txt        lsmod
sh byname.txt       ls -l /dev/block/by-name
sh blockdev.txt     ls -l /dev/block

echo "== listings =="
sh vendor_bin.txt       ls -l /vendor/bin
sh vendor_lib64.txt     ls -l /vendor/lib64
sh odm_bin.txt          ls -l /odm/bin
sh system_ext_bin.txt   ls -l /system_ext/bin
sh modules.txt          ls -lR /vendor/lib/modules
sh touch.txt            ls -lR /sys/class/touch
sh firmware_mnt.txt     ls -lR /vendor/firmware_mnt

echo "== hal binaries of interest =="
"$ADB" shell 'ls -l /vendor/bin/hw | grep -Ei "gatekeeper|keymint|keymaster|boot|secure_element|weaver|qsee|hlos"' \
    | tr -d '\r' > "$OUT/hal_bins.txt"
"$ADB" shell 'ls -l /odm/bin/hw 2>/dev/null' | tr -d '\r' > "$OUT/odm_bin_hw.txt"
cat "$OUT/hal_bins.txt"

echo
echo "== directories =="
pull /vendor/etc/init
pull /vendor/etc/vintf
pull /odm/etc/init
pull /odm/etc/vintf
pull /vendor/lib/modules
pull /odm/firmware

echo
echo "wrote to $OUT"
du -sh "$OUT" 2>/dev/null
