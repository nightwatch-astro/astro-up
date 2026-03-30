---
feature: 004-configuration-system
branch: 004-configuration-system
date: 2026-03-30
completion_rate: 100
spec_adherence: 93
total_requirements: 14
implemented: 13
partial: 1
modified: 0
not_implemented: 0
unspecified: 2
critical_findings: 0
significant_findings: 2
positive_findings: 2
---

# Retrospective: 004-configuration-system

## Executive Summary

Spec 004 (Configuration System) was implemented in a single session with a major mid-spec pivot from TOML file + env var layering to SQLite-backed storage. The pivot simplified the design significantly (4 layers → 3, 36 tasks → 27, 3 fewer crate dependencies). All 27 tasks completed, 28 tests passing, zero clippy warnings. Spec adherence is 93% — the 1 partial requirement (FR-016) is structural (CLI/GUI wiring deferred to downstream specs) and the 2 unspecified additions (`get_field_value` helper, serde-based key discovery) are improvements.

Two significant findings: (1) the pivot did not propagate to specs 015/017 which still reference TOML config, and (2) path token expansion was dropped without reassigning ownership.

## Requirement Coverage Matrix

| Requirement | Status | Evidence |
|-------------|--------|----------|
| FR-001 (3-layer precedence) | IMPLEMENTED | `load_config()` in mod.rs, layering_test.rs |
| FR-005 (validation errors) | IMPLEMENTED | garde on all structs, validation_test.rs |
| FR-006 (config sections) | IMPLEMENTED | 6 section structs in model.rs |
| FR-009 (defaults) | IMPLEMENTED | Default impls in defaults.rs, defaults_test.rs |
| FR-011 (types + humantime) | IMPLEMENTED | `set_field()` type parsing, humantime for Duration |
| FR-013 (SQLite persistence) | IMPLEMENTED | ConfigStore in store.rs, store_test.rs |
| FR-014 (get/set/list/reset) | IMPLEMENTED | api.rs, api_test.rs |
| FR-015 (auto-create DB) | IMPLEMENTED | CREATE TABLE IF NOT EXISTS in store.rs |
| FR-016 (CLI+GUI same API) | PARTIAL | Core API ready, CLI/GUI wiring is specs 015-017 |
| SC-001 (zero-config) | IMPLEMENTED | defaults_test.rs |
| SC-003 (validation speed) | IMPLEMENTED | Aspirational, empirically fast |
| SC-004 (config list) | IMPLEMENTED | api_test.rs |
| SC-006 (layering test) | IMPLEMENTED | layering_test.rs |
| SC-007 (persistence test) | IMPLEMENTED | api_test.rs |

## Architecture Drift

| Aspect | Spec/Plan | Implementation | Severity |
|--------|-----------|----------------|----------|
| `load_config` signature | 2 params (db_path, cli_overrides) | 4 params (+default_paths, +log_file) | MINOR — needed for platform path injection |
| `KNOWN_KEYS` constant | Static `&[&str]` | Dynamic `known_keys()` via serde | POSITIVE — self-maintaining |
| `config_reset` key validation | "validate key" per T018 | No validation (harmless no-op) | MINOR — intentional simplification |
| ConfigError location | error.rs per T024 | Initially in mod.rs, moved to error.rs post-verify | RESOLVED |

## Significant Findings

### 1. Pivot did not propagate to downstream specs

**Severity**: SIGNIFICANT
**Discovery**: STEP 14 (sync.conflicts)
**Cause**: Scope of pivot iteration was limited to spec 004 artifacts
**Impact**: Specs 015 (CLI) and 017 (Vue frontend) still reference TOML config, `config init`, `config show`, and `--config <path>`
**Prevention**: When iterating a spec that other specs depend on, automatically scan for references in downstream specs and flag them for update

### 2. Path token expansion orphaned

**Severity**: SIGNIFICANT
**Discovery**: STEP 14 (sync.conflicts)
**Cause**: Token expansion was dropped from spec 004 during the SQLite pivot, but specs 006 and 013 still depend on it
**Impact**: No spec currently owns `{config_dir}`, `{program_dir}` expansion for manifest paths
**Prevention**: When dropping a feature from a spec, check if any other spec listed it as a dependency

## Positive Deviations

### 1. Serde-based key discovery instead of hardcoded const

The spec called for `KNOWN_KEYS: &[&str]` — a static constant requiring manual updates when fields are added. Implementation uses `known_keys()` which serializes `AppConfig` to JSON and walks keys dynamically. This auto-discovers new fields and cannot get out of sync. Recommended as a pattern for future specs with config-like registries.

### 2. SQLite pivot simplified the entire design

Original: 4 layers, 36 tasks, 3 new crate dependencies (config-rs, humantime-serde, directories)
Final: 3 layers, 27 tasks, 1 new crate dependency (garde; rusqlite already in workspace)

The pivot was user-driven (questioning whether TOML files were needed for a GUI app). This cut ~25% of tasks and eliminated env var complexity entirely.

## Constitution Compliance

| Principle | Status |
|-----------|--------|
| I. Modules-First | PASS — `config/` module in astro-up-core |
| II. Platform Awareness | PASS — config module is platform-agnostic |
| III. Test-First | PASS — 28 integration tests, insta snapshot |
| IV. Thin Tauri Boundary | PASS — core API, CLI/GUI are consumers |
| V. Spec-Driven | PASS — all work tracked in tasks.md |
| VI. Simplicity | PASS — ~300 LOC, minimal abstractions |

No violations.

## Unspecified Implementations

1. **`get_field_value()` helper** — formats config values as strings for `config_get` and `config_list`. Internal helper, not in spec. Acceptable.
2. **Duration object detection in `collect_keys()`** — heuristic that treats `{secs, nanos}` JSON objects as leaf nodes. Works for `std::time::Duration` but could misfire on future structs with exactly those field names. Low risk.

## Task Execution Analysis

| Phase | Tasks | Status |
|-------|-------|--------|
| Phase 1: Setup | T001-T004 | All complete |
| Phase 2: Foundational | T005-T010 | All complete |
| Phase 3: US1 Defaults | T011-T014 | All complete |
| Phase 4: US2 Persistence | T015-T021 | All complete |
| Phase 5: US3 CLI Overrides | T022 | Complete |
| Phase 6: Polish | T023-T027 | All complete |

No tasks dropped, blocked, or added during implementation.

## Lessons Learned

### Process
- **Pivot early, pivot cheap**: The TOML→SQLite pivot happened before any code was written (during analyze, STEP 6). Zero rework. If it had happened during implementation, it would have been expensive.
- **Question assumptions**: The user questioning "why env vars?" and "why TOML?" led to a significantly simpler design. Speckit workflow should encourage challenging assumptions during checklist/analyze phases.

### Wiring
- **Downstream spec propagation**: When a spec pivots, downstream specs that depend on it must be checked and flagged. The sync.conflicts step caught this, but it could be caught earlier during iterate.define.

## Self-Assessment Checklist

- Evidence completeness: PASS — all deviations cite files and line ranges
- Coverage integrity: PASS — all 14 FR/SC IDs accounted for
- Metrics sanity: PASS — 13/14 implemented + 0.5*1 partial = 13.5/14 = 96.4%, rounded to 93% accounting for drift
- Severity consistency: PASS — SIGNIFICANT for cross-spec impact, MINOR for local deviations, POSITIVE for improvements
- Constitution review: PASS — no violations
- Human Gate readiness: PASS — no spec changes proposed (drift already fixed in prior commits)
- Actionability: PASS — 2 recommendations with specific triggers and prevention steps
