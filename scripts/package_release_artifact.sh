#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:?target id, e.g. darwin-arm64}"
VERSION="${2:?release version, e.g. 0.4.0}"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINARY="$ROOT_DIR/src-tauri/target/release/skill-manager"
OUT_DIR="$ROOT_DIR/.artifacts/release"
ARTIFACT="$OUT_DIR/skill-manager-v${VERSION}-${TARGET}.tar.gz"

if [[ ! -x "$BINARY" ]]; then
  echo "Release binary not found: $BINARY" >&2
  exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE/skill-manager" "$OUT_DIR"
cp "$BINARY" "$STAGE/skill-manager/skill-manager"
chmod +x "$STAGE/skill-manager/skill-manager"
tar -C "$STAGE" -czf "$ARTIFACT" skill-manager
shasum -a 256 "$ARTIFACT" | awk '{print $1 "  " FILENAME}' FILENAME="$(basename "$ARTIFACT")" > "${ARTIFACT}.sha256"
echo "Packaged $ARTIFACT"
