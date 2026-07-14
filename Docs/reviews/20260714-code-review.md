# Tauri Migration Code Review

**Date:** 2026-07-14 23:02:40  
**Range:** `4f085f8` → `e35a294` (8 commits)  
**Reviewer:** code-reviewer agent

---

## Migration Progress Summary

| Domain | Python status | Tauri/Rust status | % complete | Notes |
|--------|---------------|-------------------|------------|-------|
| **Shell / packaging** | PyInstaller + npm CLI | Tauri 2 + embedded axum server | ~70% | Native window, fixed port 18000, serves `frontend/dist` in prod |
| **Health** | `GET /api/health` | `GET /api/health` | 100% | Returns `{ "status": "ok" }` |
| **Settings** | GET + PUT harness support | GET only, schema mismatch | ~25% | Missing `supportEnabled`, camelCase fields, PUT handler |
| **Skills** | Full inventory, CRUD, enable/disable, manage, detail, source-status | GET list only, simplified read model | ~15% | No symlink sync, no unmanaged skills, no mutations |
| **MCP** | Full inventory + reconcile + adopt | Empty stub | ~5% | `GET /api/mcp/servers` returns empty arrays |
| **Slash commands** | Full CRUD + sync + review | Empty stub | ~5% | `GET /api/slash-commands` returns empty shell |
| **Marketplace** | Skills/MCP/CLI browse + install | GET stubs, placeholder details | ~20% | Routes exist; no network/cache; no `POST /install` |
| **Scan** | Config CRUD, LLM detection, skill scan | `GET /configs` empty stub | ~5% | 10+ Python endpoints missing |
| **Harness kernel** | Full catalog, bindings, support store | Install probe only | ~15% | No env overrides, no binding profiles, no enable/disable |
| **Frontend themes** | N/A (single theme) | 5-theme config-driven system | ~90% | `themes.ts`, `useTheme.tsx`, `ThemeSelector`, CSS token migration |
| **Frontend ↔ backend** | Proxy to :8000 | Tauri detection → :18000 | ~60% | HTTP fetch (no Tauri IPC); works for GETs |
| **Tests (Rust)** | N/A | 0 tests | 0% | `cargo test` runs 0 tests; README documents it anyway |
| **Tests (frontend)** | Existing vitest suite | Updated with `ThemeProvider` | ~80% | Tests still mock Python-shaped API; not run against Rust |

---

## Strengths

1. **Solid scaffolding direction** — `src-tauri/src/lib.rs` wires `AppPaths`, `HarnessKernel`, and a CQRS-shaped skills module (`store` → `read_models` → `queries`) that mirrors the Python `application/` layering.

2. **Path parity** — `src-tauri/src/paths.rs` reproduces macOS Application Support and XDG layout from the Python `paths.py`, including skills store, MCP manifest, slash-command dirs, and SQLite path.

3. **Skills page response shape** — `src-tauri/src/skills/read_models.rs` defines typed structs with `#[serde(rename = "...")]` camelCase fields matching the OpenAPI `SkillsPageResponse` contract. Recent fix commit addressed this explicitly.

4. **Frontend API resolution** — `frontend/src/api/paths.ts` cleanly branches on `__TAURI_INTERNALS__` to call `http://127.0.0.1:18000/api/...` in desktop mode vs relative URLs in browser dev mode.

5. **Theme system** — `frontend/src/lib/themes.ts` implements 5 config-driven themes with FOUC prevention via inline script in `frontend/index.html` and runtime injection. `ThemeSelector` integrates into settings and sidebar with i18n.

6. **Production serving** — `src-tauri/src/server/mod.rs` nests API under `/api` and falls back to `ServeDir` for the built SPA, matching the Python FastAPI pattern in `skill_manager/api/app.py`.

7. **Harness install detection** — `src-tauri/src/harness.rs` covers all 6 harnesses with `which` probes and skill root paths aligned with `skill_manager/harness/catalog.py` defaults.

---

## Issues

### Critical (Must Fix)

#### 1. Settings API returns wrong JSON shape for frontend

**Where:** `src-tauri/src/server/routes/settings.rs:12-23`, `src-tauri/src/harness.rs:4-11`

**What's wrong:** `HarnessStatus` serializes with serde's default snake_case (`logo_key`, `managed_location`). The frontend expects camelCase (`logoKey`, `managedLocation`) and requires `supportEnabled` (see `skill_manager/application/settings/presenters.py:43-49`, `frontend/src/features/settings/components/SettingsHarnessCard.tsx:17-30`).

**Why it matters:** Settings page harness toggles and avatars will break silently in Tauri mode — switches won't reflect persisted state, logos won't render.

**Fix:** Add `#[serde(rename_all = "camelCase")]` on `HarnessStatus`, add `support_enabled: bool` loaded from `settings.json` (or default all true), serialize `managed_location` as string.

---

#### 2. Skills harness cell states are incorrect

**Where:** `src-tauri/src/skills/read_models.rs:95-101`

**What's wrong:** Cell state is derived only from whether the harness binary is installed, not from actual symlink/inventory state:

```rust
let state = if h.installed && installed_harnesses.contains(&h.harness) {
    "enabled"
} else if h.installed {
    "disabled"
} else {
    "empty"
};
```

Every skill row shows all installed harnesses as `"enabled"`. Python uses per-skill symlink inspection via `cell_state()` in `skill_manager/application/skills/policy.py:50-57`, producing `enabled`, `disabled`, `found`, or `empty`.

**Why it matters:** The skills matrix — the core product surface — displays misleading data. Users cannot tell which harnesses actually have a skill enabled.

**Fix:** Implement inventory/sighting model; inspect harness skill directories for symlinks or canonical links per skill.

---

#### 3. All write/mutation API endpoints are missing

**Where:** Entire `src-tauri/src/server/routes/` — only GET handlers exist.

**What's wrong:** Python exposes ~40+ endpoints across domains. Rust implements 8 GET routes, zero POST/PUT/DELETE. Frontend calls that will 404 in Tauri mode include:

| Frontend call | Python route |
|---------------|--------------|
| `POST /skills/{ref}/enable` | `skill_manager/api/routers/skills.py:43` |
| `POST /skills/manage-all` | `skills.py:75` |
| `POST /marketplace/install` | `marketplace_skills.py:50` |
| `PUT /settings/harnesses/{h}/support` | `settings.py:17` |
| All MCP/slash-command/scan mutations | respective routers |

**Why it matters:** The app is read-only in Tauri mode. Enable/disable, adopt, install, sync, and scan are all non-functional — the primary workflows are broken.

**Fix:** Port mutation services domain-by-domain; wire `SkillsMutationService` (`src-tauri/src/skills/mutations.rs`) to routes as a starting point.

---

#### 4. No harness symlink / config sync layer

**Where:** Missing entirely in Rust; Python logic in `skill_manager/application/skills/`, `mcp/`, `slash_commands/`

**What's wrong:** Rust `SkillStore` (`src-tauri/src/skills/store.rs`) only lists directories under `data_dir/shared/`. It does not:

- Discover unmanaged skills in harness directories
- Create/remove symlinks on enable/disable
- Track `originHarness`, revisions, or source metadata
- Read/write `manifest.json` inventory

**Why it matters:** Skill Manager's value proposition is unified cross-harness management. Without symlink sync, the Rust backend is a directory listing tool, not a control center.

**Fix:** Port Python `inventory`, `mutations`, and harness binding profiles before claiming skills parity.

---

#### 5. Marketplace install and live data missing

**Where:** `src-tauri/src/server/routes/marketplace.rs`

**What's wrong:**

- `POST /api/marketplace/install` not implemented (frontend: `frontend/src/features/marketplace/api/client.ts:37`)
- `popular`/`search` return empty pages
- Detail handlers return hardcoded placeholder data (`skill_detail`, `mcp_detail`, `cli_detail` at lines 29-88)

**Why it matters:** Marketplace browse shows empty feeds; install button fails with 404.

**Fix:** Port marketplace query services and `reqwest`-based remote fetching (dependency already declared but unused in `Cargo.toml:26`).

---

#### 6. Server startup race condition

**Where:** `src-tauri/src/lib.rs:40-47`, `src-tauri/src/server/mod.rs:31-40`

**What's wrong:** Axum server starts in a spawned thread with no readiness signal. Tauri window loads immediately (via `beforeDevCommand: npm run dev`). Frontend may fire API requests before `TcpListener::bind` completes.

**Why it matters:** Intermittent connection failures on cold start, especially on slower machines.

**Fix:** Block Tauri window until health check passes, or expose a Tauri command that returns the bound URL after confirmation.

---

### Important (Should Fix)

#### 7. `SkillsMutationService` exists but is unwired

**Where:** `src-tauri/src/skills/mutations.rs:6-28`, `src-tauri/src/lib.rs:28-32`

**What's wrong:** `remove_skill` is implemented but never added to `AppState` or exposed via routes.

**Fix:** Add to `AppState`, create `DELETE /api/skills/{name}` route matching Python contract.

---

#### 8. Dead dependencies and no observability

**Where:** `src-tauri/Cargo.toml:23-32`

**What's wrong:** `rusqlite`, `reqwest`, `tracing`, `tracing-subscriber`, `uuid`, `chrono`, `toml` are declared but unused in source. `db_path` in `paths.rs:18` is never opened. No structured logging.

**Why it matters:** Bloats binary size; signals incomplete migration; scan config SQLite storage can't work without `rusqlite`.

**Fix:** Implement or remove each dependency incrementally; initialize `tracing-subscriber` in `lib.rs`.

---

#### 9. No Rust test coverage

**Where:** Entire `src-tauri/`; `README.md:331-332` documents `cargo test`

**What's wrong:** Zero unit or integration tests. Python backend has `scripts/test_backend.sh` with pytest coverage across domains.

**Fix:** Add tests for `SkillManifest::from_skill_md`, `AppPaths::resolve`, harness detection, and API response shape contracts.

---

#### 10. No OpenAPI schema from Rust backend

**Where:** Missing; frontend still uses Python-generated `frontend/src/api/generated.ts`

**What's wrong:** Type drift between Rust handlers (untyped `serde_json::Value`) and TypeScript client will go undetected. `npm run codegen:check` validates against Python only.

**Fix:** Generate OpenAPI from axum (e.g., `utoipa`) or maintain hand-written response types with contract tests.

---

#### 11. Harness kernel incomplete vs Python

**Where:** `src-tauri/src/harness.rs` vs `skill_manager/harness/catalog.py`

**What's wrong:** Rust kernel lacks:

- `SKILL_MANAGER_<HARNESS>_ROOT` env overrides
- Binding profiles for MCP, slash commands, skills discovery roots
- `HarnessSupportStore` (user enable/disable per harness in settings)
- App bundle probing for Cursor (Python uses more than `which`)

**Fix:** Port `HarnessDefinition` contract and `HarnessSupportStore` before settings/skills mutations can work correctly.

---

#### 12. Scan domain almost entirely absent

**Where:** `src-tauri/src/server/routes/scan.rs` (14 lines)

**What's wrong:** Python has 10 endpoints: availability, LLM detection, config CRUD, validation, active config, skill scan execution (`skill_manager/api/routers/scan.py`).

**Fix:** Port scan store (SQLite via `rusqlite`), LLM provider validation, and scan execution as a later phase.

---

#### 13. Permissive CORS and null CSP

**Where:** `src-tauri/src/server/mod.rs:69`, `src-tauri/tauri.conf.json:25-27`

**What's wrong:** `CorsLayer::permissive()` allows any origin to call the local API. CSP is `null`.

**Why it matters:** Low risk on localhost-only binding, but any local webpage could invoke the API if user visits a malicious site while the app runs.

**Fix:** Restrict CORS to `tauri://localhost` and Vite dev origin; define a minimal CSP for production builds.

---

#### 14. Skills actions always return `true`

**Where:** `src-tauri/src/skills/read_models.rs:113-116`

**What's wrong:** `canDelete`, `canManage`, `canStopManaging` are hardcoded `true` for every row. Python computes these via policy functions (`skill_manager/application/skills/policy.py:34-47`).

**Fix:** Port policy module; gate UI actions correctly.

---

### Minor (Nice to Have)

#### 15. Clippy naming warnings on Rust structs

**Where:** `src-tauri/src/skills/read_models.rs` (9 warnings from `cargo check`)

**Fix:** Use `#[serde(rename = "...")]` with snake_case field names, or `#[allow(non_snake_case)]` on response structs.

---

#### 16. Stale comment in vite.config.ts

**Where:** `vite.config.ts:28-29`

**What's wrong:** References `window.__SKILL_MANAGER_API_ORIGIN__` but implementation uses `__TAURI_INTERNALS__` in `paths.ts`.

**Fix:** Update comment to match actual detection mechanism.

---

#### 17. `SkillsQueryService::health()` is a stub

**Where:** `src-tauri/src/skills/queries.rs:17-19`

**Fix:** Remove or implement as part of a composite health check.

---

#### 18. Version mismatch

**Where:** `package.json` version `0.3.1` vs `src-tauri/Cargo.toml` / `tauri.conf.json` version `0.4.0`

**Fix:** Run `scripts/sync_version.py` or align manually.

---

## Recommendations

### Phase 1 — Make existing surfaces truthful (1-2 weeks)

1. Fix settings JSON schema (`supportEnabled`, camelCase).
2. Fix skills cell state logic with real symlink inspection.
3. Add server readiness gate before Tauri window loads.
4. Wire `SkillsMutationService` and implement enable/disable/delete routes.

### Phase 2 — Core domain port (3-4 weeks)

5. Port harness `HarnessDefinition` + `HarnessSupportStore`.
6. Port skills inventory (managed + unmanaged), manifest, and all mutation endpoints.
7. Port MCP manifest read/write and config reconciliation.
8. Port slash-command store and sync.

### Phase 3 — Marketplace + scan (2-3 weeks)

9. Implement marketplace remote fetching with `reqwest` and local cache.
10. Add `POST /marketplace/install`.
11. Port scan config SQLite store and LLM scan execution.

### Phase 4 — Production hardening (1-2 weeks)

12. Add Rust integration tests mirroring key Python pytest cases.
13. Generate or contract-test OpenAPI parity.
14. Tighten CORS/CSP; remove unused dependencies.
15. Update `npm run codegen` to target Rust schema when ready.

### Migration strategy

- **Keep both backends** until Rust passes a parity test suite against Python responses for all domains.
- Use feature flags or build targets (`tauri:dev` vs `start:dev`) during transition — already in place.
- Do not remove Python until: all frontend mutation paths work, data directories are shared and compatible, and release pipeline produces Tauri bundles.

---

## Assessment

**Ready to replace Python app?** **No**

**What works today in Tauri mode:**

- App launches as native window
- Health check
- Settings page loads (with schema bugs)
- Skills page loads managed skills from `~/Library/Application Support/skill-manager/shared/` with descriptions from SKILL.md frontmatter
- Harness install detection for settings display
- Theme switching across 5 themes
- Marketplace/MCP/slash-command/scan pages render empty shells without crashing

**What does not work:**

- Any user action (enable, disable, manage, install, sync, scan, settings toggle)
- Unmanaged skill discovery
- Accurate harness matrix cell states
- Live marketplace data

**Overall migration progress:** **~18% feature parity** (scaffolding ~45%, domain logic ~10%, mutations ~0%, tests ~0%)

**Reasoning:** The commit range delivers a credible Tauri shell with correct project structure and a polished frontend theme system, but the Rust backend is predominantly stubbed GET handlers. Only skills list query has partial real logic, and even that misrepresents harness state. The architecture direction is sound and mirrors Python CQRS patterns, but roughly 80% of business logic remains in `skill_manager/application/` with no Rust equivalent yet.
