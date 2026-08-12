#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

bash "$ROOT_DIR/scripts/test_rust.sh"
pnpm run typecheck
pnpm test
VITE_API_BASE=/api pnpm exec vite build
