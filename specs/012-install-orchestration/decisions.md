# Decisions Report: 012-install-orchestration
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions
- **Sequential installs, not parallel**: Windows installers often conflict when running simultaneously (shared MSI mutex, registry locks). Sequential is safer.
- **Continue-on-error for independent packages**: One failure shouldn't block unrelated updates. Only skip packages that depend on the failed one.
- **Post-install verification mandatory**: Always re-detect after install to confirm the version changed. Catches silent installer failures.
- **Disk space check before download**: Prevents wasted bandwidth on downloads that can't be installed.
- **No auto-downgrade**: Downgrades are dangerous for astrophotography equipment drivers. Require explicit flag.

## Questions I Would Have Asked
- Q1: Should the engine support rollback on install failure? Decision: No in v1. Rollback requires uninstall + reinstall old version, which is complex and risky. The backup (spec 013) preserves config, not the application itself.
- Q2: Should bulk updates have a maximum concurrency? Decision: Sequential only — no concurrency for installs. Downloads could be parallelized in a future optimization.
- Q3: Should the engine log to a persistent install history? Decision: Yes — log each operation to a local SQLite database (deferred to a separate small feature, not a full spec).

## Clarify-Phase Decisions

### C1: Static dependency graph from catalog, not runtime discovery
**Decision**: Building a dependency graph at runtime (scanning installed packages for dependencies) is fragile. Manifest-declared dependencies are explicit and testable. The trade-off is that manifest authors must declare all dependencies.

### C2: Backup failure is a user decision, not an auto-abort
**Decision**: Some users may prefer to update without backup (e.g., fresh install, no config to back up yet). If backup fails, present the choice: proceed without backup, or abort. Default in non-interactive mode: proceed with warning.

### C3: Dry-run produces the same plan as a real run
**Decision**: Dry-run builds the full update plan (dependency resolution, download URLs, version comparisons) and serializes it. The only difference is no side effects (no download, no install). This ensures dry-run is trustworthy.

### C4: One install at a time, no parallel installs
**Decision**: Windows installers often share global state (MSI mutex, registry, COM registration). Parallel installs would cause unpredictable failures. Sequential is the only safe approach.

### C5: Post-install verification is mandatory, not optional
**Decision**: Every install triggers a re-detection. This catches silent failures where the installer exits 0 but doesn't actually install (e.g., MSI rollback). The orchestration engine doesn't trust exit codes alone.
