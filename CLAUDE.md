# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build, Test, and Development Commands

```bash
# Initial dev setup (npm deps only)
scripts/install-dev.sh
# or: npm install

# Run the Tauri desktop app (embedded Rust API on :18000)
npm run tauri:dev
# Alias: scripts/start-dev.sh

# Frontend-only hot reload (no Tauri shell, needs running backend)
npm run dev

# Validation
npm run typecheck              # TypeScript type checking
npm run test:rust              # Rust integration tests (src-tauri)
npm test                       # Frontend tests (vitest)
npm run build                  # Production frontend build

# OpenAPI TypeScript client (from checked-in openapi.json)
npm run codegen:openapi
```

The app ships as a **Tauri desktop application** with an embedded Axum HTTP server on `http://127.0.0.1:18000`. Health: `http://127.0.0.1:18000/api/health`.

## Architecture

Skill Manager is a **local-first control center** for AI agent extensions (Skills, MCP servers, slash commands) across multiple agent harnesses. It ships as a Tauri desktop app (Rust backend + React frontend).

### Rust Backend (`src-tauri/`)

```
Tauri shell (lib.rs) → Axum server (server/) → Domain services → Harness/DB/Stores
```

- **`src-tauri/src/server/`** — Axum router serving `/api/*` routes and the built frontend SPA from `frontend/dist/`.
- **`src-tauri/src/harness/`** — Abstraction over 6 AI agent harnesses (Codex, Claude Code, Cursor, OpenCode, Hermes Agent, OpenClaw). Each harness is defined in `harness/catalog.rs` with binding profiles for skills, MCP, and slash commands.
- **`src-tauri/src/skills/`** — Skill inventory, policy, mutations, source fetch, and read models.
- **`src-tauri/src/mcp/`** — MCP manifest store, harness adapters, availability probes, and config sync.
- **`src-tauri/src/slash_commands/`** — TOML command library, sync state, and review queue.
- **`src-tauri/src/scan/`** — SQLite-backed LLM scan configs and static skill analysis.
- **`src-tauri/src/marketplace/`** — Skills, MCP, and CLI catalog clients with install tokens.
- **`src-tauri/src/db/`** — SQLite database for scan configs. Schema via `db/migrations.rs`.
- **`src-tauri/src/paths.rs`** — App-owned file paths under `~/Library/Application Support/skill-manager` (macOS) or XDG dirs (Linux).

### Application Container

`AppState` in `lib.rs` wires all domain services together. `build_app_state()` / `build_app_state_with_env()` construct the container used by Axum routes and integration tests.

### Harness Kernel

`HarnessKernelService` (`harness/mod.rs`) resolves runtime paths, checks harness installation (`which` + app bundle probing), and determines supported families. `HarnessSupportStore` persists user-enabled harnesses in settings.

### Frontend

React 19 + TypeScript, built with Vite. Feature-based organization under `frontend/src/features/`:

- `skills/` — Skills matrix, in-use/needs-review views, adoption flow
- `mcp/` — MCP server matrix, config resolution
- `slash-commands/` — Command library and sync management
- `marketplace/` — Skills, MCP, and CLI marketplace browsing with install
- `settings/` — Harness enable/disable, scan config management
- `overview/` — Dashboard with capability registry

Shared components live in `frontend/src/components/`. The API client (`api/generated.ts`) is generated from `frontend/src/api/openapi.json` via `openapi-typescript`. State management uses `@tanstack/react-query`. Design tokens are in `frontend/src/styles/tokens.css`.

### Internal Data Formats

- Skill sharing: managed skills stored in `data_dir/shared/` and symlinked into harness skill directories.
- MCP servers: normalized JSON manifest (`data_dir/mcp/manifest.json`), translated per-harness.
- Slash commands: TOML records in `data_dir/slash-commands/commands/` with content-hash sync tracking.
- Settings: `data_dir/settings.json` for harness support toggles.

### Key Conventions

- API request/response shapes use serde with camelCase JSON field names matching the frontend contract.
- Harness identity: `codex`, `claude`, `cursor`, `opencode`, `hermes`, `openclaw`.
- Extension families: `skills`, `mcp`, `slash_commands`.
- Env vars for custom skill roots: `SKILL_MANAGER_<HARNESS>_ROOT`.
- Frontend uses CSS custom properties for theming.

### Migration Notes

See `Docs/migration/parity-checklist.md` and `Docs/migration/remaining-gaps.md` for Python→Rust parity status. All 56 API routes exist; some behaviors (LLM scan, HTTP MCP probe) are partial.

### Integration Tests

```bash
cd src-tauri && cargo test --no-fail-fast -- --test-threads=1
# or: npm run test:rust
```

Tests live in `src-tauri/tests/` with shared fixtures in `tests/common/mod.rs`.
