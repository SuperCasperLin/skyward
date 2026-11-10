#!/usr/bin/env bash
# Build a Skyward recovery image for nuwa (Xiaomi 13 Pro).
#
#   bash build.sh            # full: binaries + stock image + packed image
#   bash build.sh binaries   # binaries only
#   bash build.sh pack       # reuse the binaries, refetch nothing
#
# The image is written to upstream/target/out/<version>/, the version being
# the one in VERSION: SW-v<upstream release>-nuwa-v<port revision>. The image
# carries the same name as its folder, so two copies pulled out of the tree
# can still be told apart without looking at the directory they came from.
#
# Needs: rustc + the aarch64-linux-android target, cargo-ndk, the NDK.
set -euo pipefail

export MSYS_NO_PATHCONV=1
export MSYS2_ARG_CONV_EXCL='*'

ROOT="$(cd "$(dirname "$0")" && pwd)"
SRC="$ROOT/upstream"
DEV="nuwa"
STAGE="${1:-all}"

# The repository's own version, which names the image it builds.
VERSION="$(tr -d '[:space:]' < "$ROOT/VERSION" 2>/dev/null || true)"
[ -n "$VERSION" ] || VERSION="dev"

NDK_POSIX="${ANDROID_NDK_HOME:-$ROOT/tools/android-ndk-r29}"
[ -d "$NDK_POSIX" ] || { echo "!! no NDK at $NDK_POSIX"; exit 1; }
# cargo-ndk is a Windows program: it needs C:/ style, not /c/.
export ANDROID_NDK_HOME="$(cygpath -m "$NDK_POSIX")"

# Ramdisk::encode shells out to the reference lz4 for legacy framing.
[ -x "$ROOT/tools/lz4/lz4.exe" ] && export PATH="$ROOT/tools/lz4:$PATH"

cd "$SRC"

banner() { printf '\n=== %s ===\n' "$*"; }

if [ "$STAGE" = "all" ] || [ "$STAGE" = "binaries" ]; then
    banner "skyward (aarch64-linux-android)"
    cargo ndk -t arm64-v8a --platform 30 build --release -p skyward

    banner "skyward-pack (host)"
    cargo build --release -p skyward-pack

    banner "skyward-system-init (static, aarch64)"
    RUSTFLAGS="-C target-feature=+crt-static" \
        cargo ndk -t arm64-v8a --platform 30 build --release -p skyward-system-init \
        --target-dir target/system-init
fi

if [ "$STAGE" = "all" ] || [ "$STAGE" = "pack" ]; then
    OUT="target/out/$VERSION"
    IMG="$OUT/$VERSION.img"
    mkdir -p "$OUT" "target/stock/$DEV"

    banner "stock recovery partition"
    if [ -f "target/stock/$DEV/recovery.img" ]; then
        echo "reusing target/stock/$DEV/recovery.img"
    else
        ./target/release/skyward-pack stock \
            --config "devices/$DEV.toml" \
            --out "target/stock/$DEV/recovery.img"
    fi

    banner "pack $VERSION"
    ./target/release/skyward-pack recovery \
        --stock "target/stock/$DEV/recovery.img" \
        --binary target/aarch64-linux-android/release/skyward \
        --system-init target/system-init/aarch64-linux-android/release/system-init \
        --config "devices/$DEV.toml" \
        --font resources/fonts/MiSansVF.ttf \
        --mono resources/fonts/KirkMono-Regular.ttf \
        --mono-license resources/fonts/KirkMono-OFL.txt \
        --out "$IMG"

    banner "result"
    ls -l "$IMG"
    # -t: two spaces, not the '*' Windows sha256sum puts in front of the name.
    ( cd "$OUT" && sha256sum -t "$VERSION.img" | tee "$VERSION.img.sha256" )
fi
