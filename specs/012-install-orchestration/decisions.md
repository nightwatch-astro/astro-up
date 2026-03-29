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
