# Full-Repo Review Report

**Date**: 2026-07-23
**Scope**: `src-tauri/src` (76 Rust files, ~13.7k LOC) + `frontend/src` (312 TS/TSX files, ~33k LOC incl. tests)
**Focus**: Code structure & complexity — simplifications that preserve behavior
**Total findings**: 22 (Critical: 0, High: 5, Medium: 8, Low: 9)

> Note: the deterministic `score-component.py` pre-check is for an agents/skills
> repo and does not apply to this Tauri (Rust + React) app, so it was skipped.
> This report is a structure/complexity health check instead.

## Overall assessment

The codebase is **mature and well-factored** — clear domain layering
(Tauri shell → Axum server → domain services → harness/db/stores), typed
contracts, feature-based frontend, and generated API client. There is **no
serious over-engineering**. The improvement opportunities are almost all
**duplication** (the same helper reimplemented per module/feature) and a few
**oversized functions/controllers** that mix concerns. All findings preserve
behavior.

---

## High (fix this sprint)

- **`src-tauri/src` (7 files)** : [duplication] `atomic_write` (write-tmp-then-rename)
  is reimplemented ~7 times: `mcp/adapters.rs:441`, `slash_commands/store.rs:108`,
  `slash_commands/executor.rs:161`, `slash_commands/sync_state.rs:63`,
  `slash_commands/review_resolver.rs:201`, `mcp/store.rs:170`,
  `harness/support_store.rs:113`.
  - Fix: one shared `atomic_write(path, bytes)` helper (in `paths.rs` or a small `fsutil` module); replace all call sites. ~50 LOC removed.

- **`src-tauri/src/mcp/mappers.rs:21-535`** : [duplication] The 6 `TransportMapper`
  impls repeat ~80% identical `spec_to_dict` / `dict_to_spec` bodies; only key
  names differ (`env` vs `environment`, `headers` vs `http_headers`, `type`).
  - Fix: shared `stdio_spec()` / `http_spec()` builders + a small key-mapping struct so each mapper only declares its differences. Largest LOC win. (Trait itself is justified — 6 real impls.)

- **`frontend/src/features/{skills,mcp,slash-commands}` controllers** : [duplication]
  Bulk/multi-select machinery is reimplemented in all 3 controllers
  (`use-skills-workspace-controller.ts:287-343`,
  `use-mcp-management-controller.ts:193-257`,
  `useSlashCommandsController.ts:199-270`): a `Set<string>` selection, identical
  toggle logic, a `MultiSelectAction|null` pending state, and a `Promise.allSettled`
  runner formatting `${name}: ${reason}` failures.
  - Fix: extract a `useMultiSelect<Id>()` hook. ~60 LOC removed per controller.

- **`frontend/src/features/skills/model/use-skills-workspace-controller.ts:156-245`** :
  [duplication] Every structural action exists twice, differing only by a
  `reportError` boolean (`handleManageSkill`/`…FromList`, `handleDeleteSkill`/`…FromList`,
  `handleRemoveSkill`/`…FromList`, `handleToggleSkill`/`…FromList`).
  - Fix: one function per action; pass `reportError` at the single call site. ~40 LOC removed; brings the 486-line file under ~350.

- **`src-tauri/src/skills/adapters.rs:370-467`** : [complexity] `scan_skill_roots`
  is ~100 lines, 7 parameters, deeply nested hermes-policy branching (double
  `parse_skill_package`, exclusion re-checks, source re-resolution at 440-455).
  - Fix: extract `classify_skill_root(...) -> RootDecision {Skip, Include(source)}` and a `name_candidates(...)` helper (the 4 hermes helpers share the same candidate-list pattern).

## Medium (fix when touching these files)

- **`src-tauri/src/mcp/adapters.rs:488-618` + `skills/read_models.rs:130-236`** :
  [duplication] TTL snapshot cache (`CachedSnapshot` + `Arc<Mutex<Option<..>>>` +
  1s TTL + `snapshot()`/`invalidate()`) is copy-pasted.
  - Fix: generic `TtlCache<T>` with `get_or_refresh(|| …)` / `invalidate()`.

- **`src-tauri/src/skills/adapters.rs:720` + `skills/store.rs:240`** : [duplication]
  `copy_dir_all` is duplicated verbatim.
  - Fix: move to one shared helper; import in both.

- **`src-tauri/src/harness/catalog.rs:15-69`** : [boilerplate] ~18 near-identical
  one-line path fns (`codex_config`, `claude_*`, `cursor_*`, …), each just
  `ctx.home.join(...)`.
  - Fix: a `path_fn!` macro or store path segments as `&[&str]` data resolved generically. ~55 LOC → a table.

- **`src-tauri/src/scan/harness_scanner.rs:206-266`** : [complexity]
  `invoke_harness_cli` builds per-harness args via a big `match` plus special-cased
  codex-stdin logic in two places (223-224, 248-252).
  - Fix: model each harness invocation as data (args template + `uses_stdin`) on the catalog; one generic spawn path.

- **`frontend/src/features/skills/model/skillCategory.ts`** : [dead-code]
  `categoryOrder` (:377) and `SkillCategoryDefinition` (:17) are unused (grep-verified).
  - Fix: delete both. `CATEGORY_ORDER` is still used internally at :371 — keep it.

- **`frontend/src/features/mcp/model/use-mcp-management-controller.ts:57-76`** :
  [complexity] A `useEffect` mutating a `useRef<Set>` to dedupe fire-and-forget
  availability checks (manual `${name}:${revision}` key, rollback-on-catch).
  - Fix: move "check once per revision" into the query layer keyed by revision so react-query handles dedupe/caching.

- **`frontend/src/features/slash-commands/model/useSlashCommandsController.ts:37-81`** :
  [complexity] `savedCommandSnapshot` optimistic layer + reconciliation effect
  duplicates what a react-query cache update would provide.
  - Fix: `setQueryData` on mutation success; drop the snapshot state + effect (~15 LOC, one drift source removed).

- **`frontend/src/app/capability-registry/overview.ts` (518 LOC)** : [complexity]
  Mixes ~8 interface declarations with cross-feature aggregation `useMemo`.
  - Fix: split types into `overview-types.ts`; keep assembly separate.

## Low (nice to have)

- **`src-tauri/src/mcp/adapters.rs:345-357`** : [dead-code] `ensure_subtree`'s
  `format: ConfigFileFormat` param is discarded (`let _ = format;`) and threaded
  through `set_subtree_entry` for nothing. Drop it.
- **`src-tauri/src/mcp/adapters.rs:439`** : [boilerplate] `use std::sync::LazyLock;`
  sits mid-file; move to the top import block.
- **`src-tauri/src/scan/harness_scanner.rs:268-288`** : [boilerplate]
  Hand-rolled 200ms busy-poll `wait_with_timeout`; the "120 seconds" string at :283
  duplicates `SCAN_TIMEOUT_SECS` (drift risk). At minimum interpolate the constant.
- **`src-tauri/src/marketplace/skills.rs:230-323`** : [boilerplate] Hand-rolled
  `urlencoding_encode` / `decode_unicode_escape`; replace with a crate if already
  in the dep tree (verify parity first).
- **`src-tauri/src/skills/mutations.rs` + `mcp/mutations.rs`** : [duplication]
  Repeated require→act→`invalidate()`→`json!({"ok":true})` shape; a small wrapper
  helper could dedupe the sentinel/invalidate.
- **`frontend/src/features/{skills,slash-commands,mcp}/model/use*ViewMode.ts`** :
  [boilerplate] 3 near-identical thin wrappers over `usePersistentViewMode`; a
  `makePersistentViewMode(key, modes[], default)` factory removes the hand-written
  validators.
- **`frontend` controllers** : [boilerplate] `error instanceof Error ? error.message : "…"`
  repeated ~15×; add a `toErrorMessage(error, fallback)` util.
- **`frontend/src/features/mcp/model/selectors.ts:103-121,197-208`** : [complexity]
  `pillCounts` filters `inUseEntries` 4× and recomputes `addressableHarnesses`;
  compute in one pass and pass a precomputed set into `matrixCoverage`.
- **`frontend/src/features/slash-commands/model/useSlashCommandsController.ts:295-331`** :
  [over-engineering] Returns 30+ raw state setters; expose intent-named callbacks
  like the other two controllers for a consistent controller boundary.

## Systemic Patterns

- **Reimplemented file-write helper** (7 files): `atomic_write` — one shared fs
  helper eliminates the cluster. Highest-leverage, lowest-risk.
- **Per-feature controller machinery** (3 controllers): multi-select + bulk-action
  + `error.message` extraction repeated — extract `useMultiSelect` + `toErrorMessage`.
- **Per-feature search/selectors** (3 `selectors.ts`): case-insensitive substring
  filter reimplemented — a shared `matchesQuery(parts, query)`.
- **Per-feature i18n boilerplate** (3 `i18n.ts`): acceptable — already abstracted
  via `useLocalizedCopy`; only the trailing `useXCopy()` wrapper is redundant.

## Not flagged (checked, healthy)

- `TransportMapper` trait — 6 real impls, correct abstraction (the bodies, not the
  trait, are the issue).
- Axum route modules / service layering — appropriately factored; no single-impl
  factories found.
- `skills/adapters.rs` vs `mcp/adapters.rs` — different domains (symlink FS vs
  config-file parsing), not cross-duplicated.

## Review Metadata

- Method: 2 parallel review agents (Rust backend, frontend) + direct spot-reads;
  dead-code and duplication claims grep-verified.
- Score pre-check: n/a (not an agents/skills repo).
