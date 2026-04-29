#!/usr/bin/env bash
# Build XGen Protocol and copy output binaries to bin/
set -e

PROFILE=${1:-debug}
BIN_DIR="$(dirname "$0")/bin"

if [ "$PROFILE" = "release" ]; then
    cargo build --release
    SRC="C:/cargo-targets/XGenProtocol/release"
else
    cargo build
    SRC="C:/cargo-targets/XGenProtocol/debug"
fi

mkdir -p "$BIN_DIR"
for bin in xgen-node xgen-client; do
    if [ -f "$SRC/$bin.exe" ]; then
        cp "$SRC/$bin.exe" "$BIN_DIR/$bin.exe"
        echo "Copied $bin.exe -> bin/"
    fi
done

echo "Done. Binaries in: $BIN_DIR"
