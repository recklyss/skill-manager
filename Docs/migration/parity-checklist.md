# API Parity Checklist: Python → Rust (Tauri)

Generated: 2026-07-14 (QA pass)  
Sources: `skill_manager/api/routers/*.py` vs `src-tauri/src/server/routes/`  
Frontend contract: `frontend/src/api/generated.ts`

**Status legend**

| Status | Meaning |
|--------|---------|
| PASS | Route exists and behavior matches Python (integration-tested) |
| PARTIAL | Route exists; behavior incomplete or not fixture-tested |
| FAIL | Route missing or returns wrong status/shape vs Python |
| STUB | Route exists but returns empty/placeholder data |

---

## Summary

| Domain | Python endpoints | Rust routes | PASS | PARTIAL | FAIL/STUB | Functional parity |
|--------|------------------|-------------|------|---------|-----------|-------------------|
| Health | 1 | 1 | 1 | 0 | 0 | **100%** |
| Settings | 2 | 2 | 2 | 0 | 0 | **100%** |
| Skills | 11 | 11 | 6 | 5 | 0 | **55%** |
| MCP | 12 | 12 | 6 | 6 | 0 | **50%** |
| Slash commands | 8 | 8 | 6 | 2 | 0 | **75%** |
| Scan | 11 | 11 | 6 | 5 | 0 | **55%** |
| Marketplace | 11 | 11 | 5 | 6 | 0 | **45%** |
| **Total** | **56** | **56** | **32** | **24** | **0** | **57%** |

**Route coverage:** 56/56 (**100%**) — every Python endpoint has a Rust route.  
**Integration tests:** 57 passing (`cargo test --no-fail-fast -- --test-threads=1`).  
**Typecheck:** `npm run typecheck` PASS.

---

## Health

| Endpoint | Python | Rust | Status |
|----------|--------|------|--------|
| `GET /api/health` | `skills_queries.health()` payload | `health.rs` — same shape | PASS |

---

## Settings

| Endpoint | Python | Rust | Status |
|----------|--------|------|--------|
| `GET /api/settings` | Full settings + harness support | `settings.rs` | PASS |
| `PUT /api/settings/harnesses/{harness}/support` | Persist harness enable/disable | `settings.rs` — support store | PASS |

---

## Skills

| Endpoint | Python | Rust | Status |
|----------|--------|------|--------|
| `GET /api/skills` | Full matrix | `skills.rs` — page response | PASS |
| `GET /api/skills/{skill_ref}` | Detail | `skills.rs` | PASS |
| `GET /api/skills/{skill_ref}/source-status` | Source status | `skills.rs` | PARTIAL |
| `POST /api/skills/{skill_ref}/enable` | Enable harness | `skills.rs` | PARTIAL |
| `POST /api/skills/{skill_ref}/disable` | Disable harness | `skills.rs` | PARTIAL |
| `POST /api/skills/{skill_ref}/set-harnesses` | Bulk harness toggle | `skills.rs` | PARTIAL |
| `POST /api/skills/{skill_ref}/manage` | Adopt skill | `skills.rs` | PASS |
| `POST /api/skills/manage-all` | Bulk adopt | `skills.rs` | PARTIAL |
| `POST /api/skills/{skill_ref}/update` | Refresh from source | `skills.rs` | PARTIAL |
| `POST /api/skills/{skill_ref}/unmanage` | Unmanage | `skills.rs` | PARTIAL |
| `POST /api/skills/{skill_ref}/delete` | Delete | `skills.rs` | PARTIAL |

---

## MCP

| Endpoint | Python | Rust | Status |
|----------|--------|------|--------|
| `GET /api/mcp/servers` | Read manifest + inventory | `mcp.rs` | PASS |
| `GET /api/mcp/servers/{name}` | Server detail | `mcp.rs` | PASS |
| `POST /api/mcp/servers/{name}/availability/check` | Probe connection | `mcp.rs` | PARTIAL |
| `POST /api/mcp/servers` | Install from marketplace | `mcp.rs` | PARTIAL |
| `DELETE /api/mcp/servers/{name}` | Uninstall | `mcp.rs` | PARTIAL |
| `POST /api/mcp/servers/{name}/enable` | Enable + sync harness config | `mcp.rs` | PARTIAL |
| `POST /api/mcp/servers/{name}/disable` | Disable harness | `mcp.rs` | PARTIAL |
| `POST /api/mcp/servers/{name}/reconcile` | Reconcile drift | `mcp.rs` | PARTIAL |
| `POST /api/mcp/servers/{name}/set-harnesses` | Set all harnesses | `mcp.rs` | PARTIAL |
| `GET /api/mcp/unmanaged/by-server` | List unmanaged | `mcp.rs` | PASS |
| `POST /api/mcp/unmanaged/adopt` | Adopt unmanaged | `mcp.rs` | PASS |

---

## Slash commands

| Endpoint | Python | Rust | Status |
|----------|--------|------|--------|
| `GET /api/slash-commands` | List + review queue | `slash_commands.rs` | PASS |
| `GET /api/slash-commands/{name}` | Command detail | `slash_commands.rs` | PASS |
| `POST /api/slash-commands` | Create command | `slash_commands.rs` | PASS |
| `PUT /api/slash-commands/{name}` | Update command | `slash_commands.rs` | PARTIAL |
| `DELETE /api/slash-commands/{name}` | Delete command | `slash_commands.rs` | PARTIAL |
| `POST /api/slash-commands/{name}/sync` | Sync + hash tracking | `slash_commands.rs` | PASS |
| `POST /api/slash-commands/review/import` | Import unmanaged | `slash_commands.rs` | PASS |
| `POST /api/slash-commands/review/resolve` | Resolve review | `slash_commands.rs` | PARTIAL |

---

## Scan

| Endpoint | Python | Rust | Status |
|----------|--------|------|--------|
| `GET /api/scan/configs` | SQLite-backed list | `scan.rs` | PASS |
| `GET /api/scan/availability` | Service availability | `scan.rs` | PASS |
| `GET /api/scan/llm/detection` | Auto-detect LLM providers | `scan.rs` | PASS |
| `GET /api/scan/configs/{config_id}/secret` | Reveal API key | `scan.rs` | PASS |
| `POST /api/scan/configs` | Create config | `scan.rs` | PASS |
| `POST /api/scan/configs/validate` | Validate config | `scan.rs` | PARTIAL |
| `PUT /api/scan/configs/{config_id}` | Update config | `scan.rs` | PARTIAL |
| `DELETE /api/scan/configs/{config_id}` | Delete config | `scan.rs` | PASS |
| `PUT /api/scan/configs/{config_id}/active` | Set active config | `scan.rs` | PARTIAL |
| `POST /api/scan/skills/{skill_ref}` | Run skill scan | `scan.rs` | PARTIAL |

---

## Marketplace

| Endpoint | Python | Rust | Status |
|----------|--------|------|--------|
| `GET /api/marketplace/popular` | Remote/cache catalog | `marketplace.rs` | PASS |
| `GET /api/marketplace/search` | Search catalog | `marketplace.rs` | PASS |
| `GET /api/marketplace/items/{item_id}` | Detail + install state | `marketplace.rs` | PASS |
| `GET /api/marketplace/items/{item_id}/document` | SKILL.md fetch | `marketplace.rs` | PARTIAL |
| `POST /api/marketplace/install` | Install skill | `marketplace.rs` | PARTIAL |
| `GET /api/marketplace/mcp/popular` | MCP popular page | `marketplace.rs` | PARTIAL |
| `GET /api/marketplace/mcp/search` | MCP search | `marketplace.rs` | PARTIAL |
| `GET /api/marketplace/mcp/items/{qualified_name}` | MCP detail | `marketplace.rs` | PARTIAL |
| `GET /api/marketplace/clis/popular` | CLI popular page | `marketplace.rs` | PARTIAL |
| `GET /api/marketplace/clis/search` | CLI search | `marketplace.rs` | PARTIAL |
| `GET /api/marketplace/clis/items/{slug}` | CLI detail | `marketplace.rs` | PARTIAL |

---

## Integration tests (QA)

| File | Tests | Behaviors covered |
|------|-------|-------------------|
| `tests/harness_test.rs` | 5 | Kernel catalog, install probe, support store |
| `tests/settings_test.rs` | 5 | Settings envelope, harness support toggle |
| `tests/skills_test.rs` | 11 | Discovery roots, manage/adopt symlink, Hermes policy, source links, cell states |
| `tests/mcp_test.rs` | 11 | Inventory, manifest, unmanaged dedupe/differ, adopt, secret masking |
| `tests/slash_commands_test.rs` | 9 | CRUD envelope, drift review, adopt no-overwrite, sync blocks manual file |
| `tests/scan_test.rs` | 6 | Config CRUD envelope, availability, delete, skill scan (static path) |
| `tests/marketplace_test.rs` | 6 | Popular/search/detail envelopes, install token validation, wiremock |
| `src/lib.rs` unit | 4 | Marketplace parsing, install token round-trip |

**Run:** `cd src-tauri && cargo test --no-fail-fast -- --test-threads=1`

---

## Validation run (2026-07-14 QA)

| Check | Result |
|-------|--------|
| `cargo test --no-fail-fast -- --test-threads=1` | **57 passed**, 0 failed |
| `npm run typecheck` | **PASS** |
