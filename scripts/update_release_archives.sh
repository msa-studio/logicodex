#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NAME="logicodex-v1.0.1-alpha"
OUT_DIR="/home/ubuntu"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/$NAME"
cd "$ROOT_DIR"
tar --exclude='./target' --exclude='./.git' --exclude='*.zip' --exclude='*.tar.gz' --exclude='*.sha256' -cf - . | tar -xf - -C "$STAGE/$NAME"
cd "$STAGE"
rm -f "$OUT_DIR/$NAME.zip" "$OUT_DIR/$NAME.tar.gz" "$OUT_DIR/$NAME.sha256"
zip -qr "$OUT_DIR/$NAME.zip" "$NAME"
tar --sort=name --owner=0 --group=0 --numeric-owner -czf "$OUT_DIR/$NAME.tar.gz" "$NAME"
cd "$OUT_DIR"
sha256sum "$NAME.zip" "$NAME.tar.gz" > "$NAME.sha256"
echo "Created $OUT_DIR/$NAME.zip"
echo "Created $OUT_DIR/$NAME.tar.gz"
echo "Created $OUT_DIR/$NAME.sha256"
