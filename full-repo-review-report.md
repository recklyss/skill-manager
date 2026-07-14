# Full-Repo Review Report

**Date**: 2026-07-15  
**Scope**: Tauri migration worktree (`main`, 8 commits ahead + unstaged Rust backend)  
**Files reviewed**: ~200 source files across `src-tauri/`, `frontend/`, `scripts/`  
**Total findings**: 28 (Critical: 2, High: 7, Medium: 11, Low: 8)

## Executive Summary

Skill Manager is mid-migration from a Python/FastAPI backend to a Tauri desktop app with an embedded Rust/Axum API server. **All 56 HTTP routes exist** in Rust with **57 integration tests passing** and **244 frontend tests passing**. Route coverage is 100%; behavioral parity is ~57% with documented gaps in LLM scanning, remote MCP probing, and release packaging. The architecture mirrors the Python CQRS layout and is suitable for Python removal on the Tauri-only dev path.

## Structural Health (adapted pre-check)

`score-component.py` skipped — not applicable (agents/skills repo tooling). Structural review performed instead:

| Area | Health | Grade | Key Issues |
|------|--------|-------|------------|
| `src-tauri/` Rust backend | 56/56 routes, 57 tests | B+ | Behavioral gaps in LLM scan + HTTP MCP probe; dead-code warnings |
| `frontend/` React SPA | 244 vitest tests pass | A- | Still references legacy Python dev scripts in root docs |
| `scripts/` | Mixed Python + Rust | C | Python release/CI scripts obsolete after migration |
| Python `skill_manager/` | Superseded | — | Safe to remove for Tauri path; keep gaps documented |

## Critical (fix immediately)

- **`src-tauri/src/scan/llm.rs`** : [functional] `validate_config_connectivity` and skill scan with `useLlm: true` do not perform live LLM requests; Python invokes provider APIs with structured threat findings.
  - Fix: Implement HTTP client calls to OpenAI/Anthropic validation endpoints; wire `LLMAnalyzer` equivalent for skill scans.

- **`.github/workflows/release.yml`** : [distribution] Release pipeline still builds PyInstaller artifacts via Python; Tauri `tauri:build` is the new distribution path but not wired in CI.
  - Fix: Replace `build_release.py` job with Rust toolchain + `npm run tauri:build`; update Homebrew/npm wrapper validation for `.dmg`/`.AppImage` artifacts.

## High (fix this sprint)

- **`src-tauri/src/mcp/availability.rs:37-48`** : [functional] HTTP/SSE MCP transports always return `unavailable`; Python performs TCP/JSON-RPC probing.
  - Fix: Add TCP connect + optional JSON-RPC handshake probe with timeout.

- **`Docs/migration/remaining-gaps.md`** : [coverage] MCP/skills mutation routes lack offline integration tests (enable, disable, reconcile, uninstall).
  - Fix: Add fixture-based tests mirroring `skills_test.rs` / `mcp_test.rs` patterns.

- **`package.json`** : [devx] `dev:backend`, `codegen:openapi`, `build:release` still reference Python `.venv`.
  - Fix: Point dev to `tauri:dev`; drop or replace OpenAPI codegen with checked-in `openapi.json`.

- **`CLAUDE.md` / `README.md`** : [docs] Architecture section describes Python FastAPI as primary backend.
  - Fix: Document Tauri + Rust Axum as primary; note behavioral gaps.

- **`src-tauri/src/skills/read_models.rs`** : [style] ~30 `non_snake_case` warnings for camelCase JSON fields.
  - Fix: Add `#[serde(rename = "...")]` on snake_case Rust fields or `#[allow(non_snake_case)]` on API response structs.

- **`skill_manager/VERSION` vs `package.json`** : [versioning] VERSION file reads `0.3.1` while `package.json`/`Cargo.toml` read `0.4.0`.
  - Fix: Single `VERSION` at repo root; sync via Node script.

- **`.github/workflows/ci.yml`** : [ci] `backend-compat` matrix runs Python 3.11–3.14 tests against removed backend.
  - Fix: Replace with `cargo test` job in `src-tauri/`.

## Medium (fix when touching these files)

- **`src-tauri/src/lib.rs`** : [architecture] `AppState` is a large god-container wiring all domains; acceptable for now but will grow.
  - Fix: Consider per-route extension traits if domains multiply.

- **`src-tauri/src/mcp/store.rs:97`** : [lint] Unused `mut` binding.
- **`src-tauri/src/server/routes/scan.rs:3`** : [lint] Unused `delete` import.
- **`src-tauri/src/harness/catalog.rs:384`** : [dead-code] `harness_definitions_for_family` unused.
- **`src-tauri/tests/common/mod.rs`** : [test-harness] Many helper methods unused across test files; consolidate or mark `#[allow(dead_code)]`.
- **`frontend/src/features/marketplace/`** : [ux] Installed-state UX recently added; ensure reinstall path matches Rust `installation.rs` states.
- **`src-tauri/gen/schemas/`** : [build] Generated Tauri ACL schemas untracked; add to `.gitignore` explicitly or commit if required by CI.
- **Marketplace remote routes** : [testing] MCP/CLI catalog browse routes have no offline fixture tests (skills has wiremock coverage).
- **Slash commands PUT/DELETE** : [testing] Routes exist; no dedicated fixture tests (low risk).
- **OpenAPI drift** : [contract] `frontend/src/api/generated.ts` generated from Python OpenAPI; no automated regen after Rust migration.
- **Packaging `npm` wrapper** : [distribution] `packaging/npm` still expects PyInstaller tarball artifacts.

## Low (nice to have)

- Rust compiler warnings for unused imports in `mcp/mod.rs`, `mcp/identity.rs`, `mcp/queries.rs`.
- `ResolvedRoot.label` field never read in `skills/adapters.rs`.
- `McpBinding.name` field never read in `mcp/contracts.rs`.
- `FileBackedMcpAdapter.context` field never read.
- Frontend `npm run dev` still starts Vite-only without Tauri shell (useful for UI-only work).
- `.nvmrc` untracked; consider committing for Node version pinning.
- IDE config dirs (`.claude/`, `.cursor/`, `.gemini/`) untracked — do not commit.

## Systemic Patterns

- **CamelCase API structs in Rust**: Response models use JavaScript field names as Rust identifiers (`logoKey`, `skillRef`) instead of `#[serde(rename)]` — seen in `skills/read_models.rs`, `settings.rs`. Fix: standardize on snake_case Rust + serde rename.

- **Partial behavioral parity with full route coverage**: 24/56 endpoints marked PARTIAL in parity checklist — routes return correct shapes but lack full Python behavior or fixture tests. Fix: prioritize LLM scan and MCP probe, then expand integration tests.

- **Python script references in npm/package scripts**: 6 root `package.json` scripts still invoke `.venv/bin/python`. Fix: batch-replace during Python removal commit.

## Python Removal Assessment

| Criterion | Status |
|-----------|--------|
| All API routes exist | ✅ 56/56 |
| Core flows work (skills, MCP, slash commands, settings) | ✅ Integration-tested |
| LLM scan parity | ❌ Static heuristics only |
| Release packaging parity | ❌ PyInstaller → Tauri transition incomplete |
| Frontend tests pass against contract | ✅ 244 pass |

**Recommendation**: Remove `skill_manager/`, `pyproject.toml`, `requirements.txt`, and Python CI jobs for the **Tauri-only dev path**. Document remaining behavioral gaps in `Docs/migration/remaining-gaps.md`. Keep `frontend/src/api/openapi.json` as the API contract source until Rust OpenAPI export exists.

## What to Delete (Python)

| Path | Reason |
|------|--------|
| `skill_manager/` | Replaced by `src-tauri/src/` |
| `tests/` (Python unit/integration) | Replaced by `src-tauri/tests/` |
| `pyproject.toml`, `requirements.txt` | No longer needed |
| `scripts/test_backend.sh`, `scripts/build_release.py`, `scripts/dump_openapi.py` | Python-specific |
| `scripts/start-dev.sh`, `scripts/stop-dev.sh` | Python managed server |
| `scripts/sync_version.py` | Replace with Node `scripts/sync_version.mjs` |

**Keep**: `frontend/`, `src-tauri/`, `scripts/test_rust.sh`, `scripts/dev-tauri.sh`, `scripts/check_release_targets_js.cjs`, marketplace wiremock fixtures inline in Rust tests.

## Test Results (2026-07-15)

| Command | Result |
|---------|--------|
| `cargo test --no-fail-fast -- --test-threads=1` | **57 passed**, 0 failed |
| `npm run typecheck` | **PASS** |
| `npm test` | **244 passed** (58 files), 0 failed |
| `bash scripts/test_rust.sh` | **57 passed**, 0 failed |

## Review Metadata

- Waves executed: structural scan + domain review + test validation
- Score pre-check: skipped (not applicable)
- Migration parity docs: `Docs/migration/parity-checklist.md`, `Docs/migration/remaining-gaps.md`
