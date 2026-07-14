# Remaining Gaps vs Python Backend

Last updated: 2026-07-14 (post QA pass)

All API routes exist (56/56). Items below are **behavioral** gaps still missing or incomplete relative to the Python implementation.

---

## Scan

1. **LLM skill analyzer** — `POST /api/scan/skills/{skill_ref}` with `useLlm: true` runs static heuristics only; Python invokes `LLMAnalyzer` with provider config, consensus runs, and structured threat findings.
2. **Config connectivity validation** — `POST /api/scan/configs/validate` checks required fields but does not call the LLM provider; Python performs a live validation request.

---

## MCP

3. **HTTP/SSE availability probe** — `POST /api/mcp/servers/{name}/availability/check` returns `unavailable` for remote transports without TCP/JSON-RPC probing; Python performs connection checks.
4. **Mutation fixture coverage** — enable, disable, reconcile, set-harnesses, uninstall, and marketplace install routes are implemented but lack offline integration tests.

---

## Marketplace

5. **Install end-to-end** — `POST /api/marketplace/install` validates tokens; full source-fetch + ingest path is not covered by fixture tests (requires mocked `SourceFetchService` or wiremock GitHub).
6. **MCP/CLI catalog tests** — browse/search/detail routes call remote registries; no offline fixture tests (skills marketplace has partial live-network coverage).

---

## Skills

7. **Mutation fixture coverage** — enable, disable, set-harnesses, manage-all, update, unmanage, and delete are implemented but only manage/adopt and body-validation paths are integration-tested.
8. **Source update** — `POST /api/skills/{skill_ref}/update` depends on live GitHub/source fetch; no offline fixture test.

---

## Slash commands

9. **PUT/DELETE** — update and delete routes exist; no dedicated fixture tests (low risk — store layer is straightforward).

---

## Non-API (out of HTTP parity scope)

- Tauri native shell lifecycle (`start`/`stop`/`status` CLI parity with Python `skill_manager` CLI)
- PyInstaller / Homebrew distribution path vs Tauri bundle
- Frontend vitest suite not run against Rust backend in CI
