#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== Building frontend ==="
VITE_API_BASE=/api npx vite build

echo ""
echo "=== Building Rust backend ==="
cd src-tauri
cargo build

echo ""
echo "=== Starting Tauri dev mode ==="
echo "The Skill Manager desktop app should open automatically."
echo "If it doesn't, run: cd src-tauri && cargo run"
echo ""
cd "$ROOT_DIR"
npx tauri dev
