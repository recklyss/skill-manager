#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?version}"
ARTIFACT_ROOT="${2:-.artifacts/release}"
TEMPLATE="packaging/homebrew/skill-manager.rb.tmpl"
OUTPUT="${3:-packaging/homebrew/skill-manager.rb}"

find_artifact() {
  local name="$1"
  find "$ARTIFACT_ROOT" -type f -name "$name" | head -n 1
}

sha_for() {
  local file="$1"
  local checksum_file
  checksum_file="$(find "$ARTIFACT_ROOT" -type f -name "$(basename "$file").sha256" | head -n 1)"
  if [[ -n "$checksum_file" && -f "$checksum_file" ]]; then
    awk '{print $1}' "$checksum_file"
    return
  fi
  shasum -a 256 "$file" | awk '{print $1}'
}

BASE_URL="https://github.com/recklyss/skill-manager/releases/download/v${VERSION}"
ARM64_FILE="$(find_artifact "skill-manager-v${VERSION}-darwin-arm64.tar.gz")"
X64_FILE="$(find_artifact "skill-manager-v${VERSION}-darwin-x64.tar.gz")"

if [[ -z "$ARM64_FILE" || -z "$X64_FILE" ]]; then
  echo "Missing macOS release tarballs for v${VERSION} under ${ARTIFACT_ROOT}" >&2
  exit 1
fi

ARM64_URL="${BASE_URL}/$(basename "$ARM64_FILE")"
X64_URL="${BASE_URL}/$(basename "$X64_FILE")"
ARM64_SHA256="$(sha_for "$ARM64_FILE")"
X64_SHA256="$(sha_for "$X64_FILE")"

sed \
  -e "s/__VERSION__/${VERSION}/g" \
  -e "s|__ARM64_URL__|${ARM64_URL}|g" \
  -e "s|__X64_URL__|${X64_URL}|g" \
  -e "s/__ARM64_SHA256__/${ARM64_SHA256}/g" \
  -e "s/__X64_SHA256__/${X64_SHA256}/g" \
  "$TEMPLATE" > "$OUTPUT"

echo "Wrote ${OUTPUT}"
