# Full-Repo Review Report — Tauri Migration

**Timestamp**: 2026-07-15 09:44:02  
**Branch**: `main` (20 commits ahead of `origin/main`)  
**Merge base**: `4f085f8193700ceb4fbc3b569f463bb409e3bcaa`  
**HEAD**: `01279412ac8e890948dd97d7cc826822666f693d`  
**Scope**: Full Python→Tauri migration + Python removal (378 files, +22,767 / −30,784 lines)

## Summary

The branch completes the planned migration from Python/FastAPI to **Tauri 2 + embedded Rust/Axum** on `127.0.0.1:18000`. Python backend (`skill_manager/`, `pyproject.toml`, Python tests) is removed. CI and release workflows now run `cargo test` and `npm run tauri:build`. All **56 HTTP routes** are registered in Rust; **57 Rust integration tests** and **244 frontend vitest tests** pass locally.

Core domains (skills inventory/symlinks, MCP manifest sync, slash commands, marketplace browse, settings, harness kernel) are implemented with architecture mirroring the former Python CQRS layout. Documented **behavioral parity gaps** remain in LLM scanning, HTTP MCP probing, and npm/Homebrew artifact packaging.

---

### Strengths

- **Complete route surface**: All API routers exist under `src-tauri/src/server/routes/` — skills (11), MCP (10), slash commands (7), scan (9), marketplace (10), settings (2), health (1) = 56 endpoints.
- **Python fully removed**: `skill_manager/`, `pyproject.toml`, `requirements.txt`, and Python CI matrix deleted; `scripts/start-dev.sh` delegates to `npm run tauri:dev`.
- **Solid test coverage for core flows**: Integration tests cover skills adopt/manage, MCP install/list/redaction, slash command sync/review, scan config CRUD, marketplace wiremock, harness resolution, and settings toggles (`src-tauri/tests/`).
- **Symlink safety**: `src-tauri/src/skills/adapters.rs` canonicalizes paths, refuses to delete non-symlink directories, and validates symlink targets before overwrite — matches Python safety model.
- **MCP secret handling**: URL query params, env vars, and headers are redacted in API responses (`src-tauri/src/mcp/redaction.rs`); integration test asserts `api_key=%5Bredacted%5D`.
- **Path policy for slash commands**: `src-tauri/src/slash_commands/path_policy.rs` normalizes `..` components and rejects paths outside harness output dirs.
- **Local-first networking**: Axum binds `127.0.0.1:18000` only (`src-tauri/src/lib.rs:32`); appropriate for a desktop control plane.
- **CI/release migrated**: `.github/workflows/ci.yml` runs Rust + frontend validation; `release.yml` builds Tauri bundles on tag push.
- **Frontend themes + marketplace UX**: Multi-theme system (`frontend/src/lib/themes.ts`), installed/reinstall states for marketplace cards, Tauri API origin detection (`frontend/src/api/paths.ts`).
- **Version sync**: Root `VERSION` (0.4.0) aligned with `package.json` and `tauri.conf.json`; `scripts/sync_version.mjs --check` in CI.
- **Docs excluded from git**: `Docs/` in `.gitignore`; commit `0127941` stops tracking migration docs.

---

### Issues

#### Critical

| # | Location | Issue |
|---|----------|-------|
| C1 | `src-tauri/src/scan/llm.rs:46-69` | `validate_config_connectivity` returns `ok: true` after field checks only — **no live LLM request**. Users see "Connectivity test passed" without provider reachability validation. |
| C2 | `src-tauri/src/scan/service.rs:84-92` | `scan_skill` with `useLlm: true` runs **static heuristics only** but reports analyzers `["static_analyzer", "llm_analyzer"]` — misleading; README promises "LLM-backed security checks". |
| C3 | `packaging/npm/scripts/release-targets.js:24-25` + `.github/workflows/release.yml:49` | **Artifact format mismatch**: npm wrapper downloads `skill-manager-v{version}-{target}.tar.gz` (PyInstaller layout); release CI uploads Tauri bundles (`.dmg`, `.AppImage`, `.deb`). Tagged releases will not satisfy `npm install @mode-io/skill-manager`. |

#### Important

| # | Location | Issue |
|---|----------|-------|
| I1 | `src-tauri/src/mcp/availability.rs:37-48` | HTTP/SSE MCP transports always return `unavailable`; comment says probing deferred. Python performed TCP/JSON-RPC checks. |
| I2 | `Docs/migration/remaining-gaps.md` (local) | Skills/MCP mutation routes (enable, disable, reconcile, uninstall, manage-all) lack offline integration tests — implemented but untested beyond happy-path adopt. |
| I3 | `frontend/src/api/generated.ts` | Generated from checked-in `openapi.json` (Python-era contract); no automated regen from Rust. Risk of silent request/response drift. |
| I4 | `src-tauri/tauri.conf.json:25-27` | `"csp": null` — no Content Security Policy for the embedded webview. |
| I5 | `.gitignore` | `src-tauri/gen/` (Tauri ACL schemas) is untracked but not ignored — will pollute `git status` on every `tauri dev/build`. |
| I6 | `src-tauri/src/server/mod.rs:89` | `CorsLayer::permissive()` on localhost-only server — low risk today, but any future `0.0.0.0` bind would expose all origins. |
| I7 | `packaging/npm/` + Homebrew | Distribution wrappers not updated for Tauri native binaries; parity doc item still open. |

#### Minor

| # | Location | Issue |
|---|----------|-------|
| M1 | `src-tauri/src/skills/read_models.rs` | ~30 `non_snake_case` warnings for camelCase JSON field names as Rust identifiers. |
| M2 | `src-tauri/src/server/routes/scan.rs:3` | Unused `delete` import. |
| M3 | `src-tauri/src/mcp/store.rs` | Unused `mut` binding (compiler warning). |
| M4 | `src-tauri/src/harness/catalog.rs:384` | `harness_definitions_for_family` dead code. |
| M5 | `src-tauri/tests/common/mod.rs` | Many test helper methods unused across files. |
| M6 | `.nvmrc` (untracked) | Pins Node **24** while CI uses Node **20** — inconsistent if committed. |
| M7 | `src-tauri/src/scan/config_service.rs:31-37` | `reveal_secret` returns full API key with no additional gate — acceptable for localhost desktop app, but worth documenting. |

---

### Recommendations

1. **Before tagging v0.4.x release**: Resolve C3 — either (a) add a CI step that packages Tauri binaries into the existing `tar.gz` layout expected by `packaging/npm/scripts/install.js`, or (b) rewrite npm/Homebrew install scripts for `.dmg`/`.AppImage`/`.deb` artifacts.
2. **LLM scan honesty (C1/C2)**: Implement live provider validation in `validate_config_connectivity`, wire real LLM analysis in `scan_skill`, or **downgrade UI copy** and remove `llm_analyzer` from the analyzers list until implemented.
3. **MCP HTTP probe (I1)**: Add TCP connect + optional JSON-RPC handshake with timeout in `probe_http`.
4. **Expand mutation tests (I2)**: Add fixture-based tests for skills enable/disable/delete and MCP reconcile/uninstall following `skills_test.rs` patterns.
5. **Hygiene**: Add `src-tauri/gen/` to `.gitignore`; fix or drop `.nvmrc` to match CI Node 20; clean Rust warnings (`#[serde(rename)]` on API structs).
6. **Contract**: Export OpenAPI from Rust or add a CI check that `openapi.json` matches route handlers.
7. **Merge strategy**: Safe to merge for **Tauri dev path** and daily use; block **public release/npm publish** until C3 is fixed.

---

### Assessment

**Ready to merge? — With fixes**

| Criterion | Status |
|-----------|--------|
| Replace Python app with Tauri | ✅ Done |
| Feature parity: detection, adopt/manage, symlinks | ✅ Core flows tested |
| Feature parity: marketplace browse/install | ✅ Browse + token validation; E2E install undertested |
| Feature parity: LLM scan | ❌ Static-only; misleading success messages |
| Feature parity: MCP HTTP availability | ❌ Stubbed unavailable |
| Tests (57 Rust + 244 frontend) | ✅ Pass |
| CI migrated to Rust/Tauri | ✅ |
| Release/npm distribution | ❌ Artifact format mismatch (C3) |
| No Docs/ in commits | ✅ `.gitignore` + untracked |

**Verdict**: Merge to `main` for the Tauri-only development and desktop use path. **Do not cut a public release or publish npm** until distribution packaging (C3) and LLM scan behavior (C1/C2) are addressed or explicitly documented as known limitations in release notes.

---

## Commits Reviewed (`origin/main..HEAD`)

```
0127941 chore: stop tracking Docs folder
922aca3 chore: remove Python backend and migrate CI to Rust
b452e4b docs: migration parity checklist and full-repo review
1185035 chore: update Tauri app icons
8521221 feat(frontend): marketplace installed state and MCP install UX
1a8e56c test: Rust integration test suite across all domains
b475e9b feat(tauri): wire API routes and application container
32b03c2 feat(tauri): marketplace catalogs and scan services
3b12602 feat(tauri): slash commands domain with sync and review queue
cb393ca feat(tauri): MCP domain with manifest sync and harness adapters
35b291b feat(tauri): skills domain with inventory, policy, and mutations
61f0905 feat(tauri): harness kernel, database, and path resolution
e35a294 fix: add all marketplace route handlers (detail + document)
25621c2 fix: skills API returns correct SkillsPageResponse format
bcd8dd8 fix: all API routes return proper JSON responses + fix tokio socket
1a8e860 fix: use fixed port 18000 for Rust backend + sync Tauri detection
f24c0a7 feat: Tauri desktop app + frontend redesign
82c0b07 fix: reliable API URL resolution via Tauri IPC
d184f00 feat: migrate to Tauri desktop app with Rust backend
ef60cff feat: config-driven multi-theme system with 5 themes
```

## Test Results (2026-07-15)

| Command | Result |
|---------|--------|
| `bash scripts/test_rust.sh` | **57 passed**, 0 failed |
| `npm run typecheck` | **PASS** |
| `npm test` | **244 passed** (58 files), 0 failed |
