# Decisions Report: 012-install-orchestration

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: Sequential installs, not parallel
**Choice**: One install at a time. Windows installers conflict (shared MSI mutex, registry locks).

### D2: Continue-on-error for independent packages
**Choice**: One failure doesn't block unrelated updates. Only skip dependents.

### D3: Post-install verification mandatory
**Choice**: Always re-detect after install. Catches silent installer failures.

### D4: No auto-downgrade
**Choice**: Reject unless `--allow-downgrade` explicitly passed. Downgrades are dangerous for drivers.

## Clarify-Phase Decisions (Interactive)

### C1: Port ParsedVersion from manifests repo — no new crate
**Finding**: User asked about version comparison for different formats. No existing crate handles all four formats (semver, date, 4-part, custom regex).
**Decision**: Port the ~150 lines of `ParsedVersion` from `astro-up-manifests/crates/shared/src/version.rs` into `astro-up-core`. Both repos maintain their own copy. The code is small and stable — shared crate overhead isn't worth it.
**Research**: Evaluated `version-compare` (45M downloads, no date support), `versions` (7.6M, no date), `lenient_semver` (unmaintained). None handle all four formats. Custom code is the right call.

### C2: Replace lenient_semver with ported lenient parser
**Finding**: `lenient_semver` hasn't been updated since 2021.
**Decision**: The ported `ParsedVersion::parse_semver()` already uses `lenient_semver` internally. When porting, we can inline the lenient logic (strip prefix, pad missing parts, strip 4th component) and drop the `lenient_semver` dependency.

### C3: Detect "newer than catalog" as distinct status
**Finding**: User's installed version could be newer than catalog (beta/nightly, self-update).
**Decision**: Add `PackageStatus::NewerThanCatalog` — distinct from `UpToDate`. Shown differently in UI (e.g., gray "dev build" badge instead of green "up to date").

### C4: Operation history in SQLite — cheap, append-only
**Finding**: User asked about cost of operation history. ~10 lines of Rust, one INSERT per operation.
**Decision**: Add `operations` table to the local SQLite database. Records: package_id, operation type (install/update/uninstall), from/to version, status (success/failed/cancelled/reboot_pending), duration_ms, error_message, created_at. No UI in this spec — data is there for future `history` command or activity log.

### C5: version_format from catalog drives parsing
**Finding**: The catalog carries `version_format` per package (from the manifest TOML). The client uses this to select the correct parser.
**Decision**: `None` or `"semver"` → semver with lenient coercion. `"date"` → chronological YYYY.MM.DD. Regex string → custom capture group comparison. This matches the manifests repo's existing behavior.

### C6: Per-package + --all is sufficient scope
**Finding**: User confirmed no `--category` filter for updates.
**Decision**: `update nina phd2` for specific packages, `update --all` for everything. No category filtering on updates.

### C7: Ledger focused on install state, history in operations table
**Finding**: User asked whether ledger makes sense for everything.
**Decision**: Ledger = "what's installed and where" (package_id, version, path, timestamp). Operations = "what happened and when" (install/update/uninstall records). Two separate concerns, two separate tables in the same SQLite database.

## Questions I Would Have Asked

### Q1: Should we support rollback on install failure?
**My decision**: No for v1. Complex and risky. Backup preserves config. If install fails, user re-runs.

### Q2: Should bulk updates have a confirmation prompt?
**My decision**: Yes — `update --all` shows the plan and asks for confirmation before proceeding. `--yes` flag skips confirmation. `--dry-run` shows plan without prompt.
