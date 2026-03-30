# Requirements Quality Checklist: Configuration System

**Purpose**: Thorough requirements quality gate before plan generation — tests completeness, clarity, consistency, and config-rs compatibility
**Created**: 2026-03-30
**Feature**: specs/004-configuration-system/spec.md
**Depth**: Thorough
**Audience**: Author (pre-plan gate)

## Requirement Completeness

- [x] CHK001 Are all configuration sections from FR-006 (catalog, paths, network, updates, logging, telemetry) fully defined with every field listed in Key Entities? — RESOLVED: Key Entities now lists exact field names per section, aligned with defaults table.
- [x] CHK002 Are cross-spec config fields documented? — RESOLVED: FR-006 now notes downstream specs (010, 013, 016) will add fields, and Dependencies section documents the relationship.
- [x] CHK003 Is the `config show` command's output format specified? — RESOLVED: FR-008 now specifies effective config with all layers merged, tokens expanded to absolute paths.
- [x] CHK004 Is the `config init` behavior specified for when a config file already exists? — RESOLVED: FR-008 and Edge Cases now specify: error with message, `--force` to overwrite.
- [x] CHK005 Are all supported CLI arguments that override config listed? — N/A for spec: `--config`, `--verbose`, `--dry-run` are examples. Exhaustive CLI args belong in spec 015 (CLI interface). FR-001 establishes CLI as highest precedence.
- [x] CHK006 Is the behavior for `config init` output specified? — RESOLVED: FR-008 now specifies fully-documented TOML with all settings commented out, explanatory comments per section.
- [x] CHK007 Is the config directory creation behavior fully specified? — RESOLVED: Edge Cases already state "creates it on first write (e.g., when saving defaults)". Read operations do not create directories.

## Requirement Clarity

- [x] CHK008 Is "sensible defaults" (FR-009) fully replaced by the explicit defaults table? — RESOLVED: FR-009 now references "the defaults documented in the Default Values table" explicitly.
- [x] CHK009 Is "clear validation error" (US2-AS3) quantified? — RESOLVED: FR-005 now specifies format: `"config.{section}.{field}: {constraint}, got {actual_value}"`.
- [x] CHK010 Is "cache TTL" semantics defined? — RESOLVED: CatalogConfig in Key Entities now specifies "hard expiry — re-fetch after TTL, no stale-while-revalidate".
- [x] CHK011 Is the `check_interval` distinguished from `cache_ttl`? — RESOLVED: UpdateConfig now clarifies "how often to check for astro-up self-updates — distinct from catalog `cache_ttl` which controls catalog freshness".
- [x] CHK012 Is "duration" type format specified? — RESOLVED: FR-011 now specifies human-readable strings via `humantime-serde` (e.g., `"24h"`, `"30s"`, `"500ms"`).
- [x] CHK013 Is the unknown env var vs TOML key asymmetry documented? — RESOLVED: FR-002 now documents that unknown `ASTROUP_` env vars are silently ignored and explains why (env vars may be set by unrelated processes).
- [x] CHK014 Is the `offline` flag scope defined? — RESOLVED: CatalogConfig now specifies "skip catalog network requests only — does not affect downloads or version checks in other specs".

## Requirement Consistency

- [x] CHK015 Is `user_agent` missing a default? — RESOLVED: Added `network.user_agent` = `astro-up/{version}` to defaults table.
- [x] CHK016 Is the public key configurable or hardcoded? — RESOLVED: Key Entities now states "catalog signature verification public key is hardcoded, not configurable" with rationale.
- [x] CHK017 Is FR-006 updated to remove GitHub token per Decision C3? — RESOLVED: FR-006 now lists `network (proxy, timeout, user agent)` — no GitHub token.
- [x] CHK018 Are `log_file_path` vs `log_file` field names consistent? — RESOLVED: Key Entities now uses `log_file` consistently, matching defaults table.
- [x] CHK019 Where does the log file path live? — RESOLVED: `log_file` is under LogConfig only. PathsConfig lists `download_dir`, `cache_dir`, `data_dir` only.

## Acceptance Criteria Quality

- [x] CHK020 Is SC-003's "100ms of startup" measurable? — KEPT AS-IS: 100ms is an aspirational target. Benchmark details are plan-phase work. The constraint is clear enough to test against.
- [x] CHK021 Is SC-005 round-trip test detailed enough? — RESOLVED: SC-005 now clarifies that round-trip tests pre-expansion config (tokens preserved), and `config show` outputs expanded paths.
- [x] CHK022 Are validation error messages specified with a consistent format? — RESOLVED: Covered by CHK009/FR-005 update.
- [x] CHK023 Is there a layering precedence acceptance criterion? — RESOLVED: Added SC-006 with explicit 4-layer precedence test.

## Scenario Coverage

- [x] CHK024 Are requirements defined for valid TOML but semantically wrong values? — RESOLVED: FR-005 now explicitly covers type mismatches (e.g., `timeout = true`).
- [x] CHK025 Are requirements for `--config` + env var interaction defined? — RESOLVED: Added to Edge Cases: env vars always apply on top of any file, layering is always CLI → env → file → defaults.
- [x] CHK026 Is `config show` with no file specified? — RESOLVED: Added to Edge Cases: shows effective config (compiled defaults + env var overrides).
- [x] CHK027 Are concurrent access requirements defined? — RESOLVED: Assumptions now state config is loaded once at startup, immutable after. No file locking needed.

## Edge Case Coverage

- [x] CHK028 Is behavior for wrong value types specified? — RESOLVED: Covered by FR-005 update (type mismatches reported same way as constraint violations).
- [x] CHK029 Are non-ASCII/spaces in paths addressed? — RESOLVED: Added to Edge Cases: platform directories crate handles natively, no special treatment.
- [x] CHK030 Is empty config file behavior specified? — RESOLVED: Added to Edge Cases: valid TOML with no keys, treated as no overrides.
- [x] CHK031 Is unresolvable config token behavior specified? — RESOLVED: Added to Edge Cases: report unresolvable token and field, exit non-zero.

## Figment Compatibility

- [x] CHK032 Is unknown TOML key detection strategy documented? — RESOLVED: FR-010 implementation note suggests two-pass approach or `deny_unknown_fields`.
- [x] CHK033 Is path token expansion post-processing documented? — RESOLVED: FR-012 implementation note documents post-config-rs-merge, pre-validation step.
- [x] CHK034 Is the CLI-to-config-rs bridge documented? — RESOLVED: FR-001 implementation note documents `Serialized::defaults(cli_overrides)` as final merge layer.
- [x] CHK035 Is double underscore nesting and case sensitivity documented? — RESOLVED: FR-002 now specifies `__` as only nesting delimiter, single `_` is literal, and notes config-rs lowercases after prefix stripping.

## Dependencies & Assumptions

- [x] CHK036 Is platform path resolution strategy specified? — KEPT AS-IS: Assumptions mention `directories` crate. Specific crate choice is implementation detail for the plan.
- [x] CHK037 Is "no secrets" constraint in the spec itself? — RESOLVED: Assumptions now explicitly state "No secrets stored in config for v1 — no GitHub token, no keychain integration."
- [x] CHK038 Is the catalog URL dependency on spec 020 documented? — RESOLVED: New Dependencies section documents the relationship.

## Notes

- All 38 items resolved. Spec updated to address all findings.
- 5 items kept as-is with rationale (plan-phase concerns or already adequate).
- Ready for STEP 4 (plan).
