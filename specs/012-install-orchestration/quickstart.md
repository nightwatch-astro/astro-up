# Quickstart: Install Orchestration Engine

## What this spec adds

An `engine` module in `astro-up-core` that coordinates the full update lifecycle:
version comparison → download → backup → install → verify.

## Key files

| File | Purpose |
|------|---------|
| `crates/astro-up-core/src/engine/mod.rs` | Module root, re-exports |
| `crates/astro-up-core/src/engine/orchestrator.rs` | Main pipeline coordinator |
| `crates/astro-up-core/src/engine/planner.rs` | Version compare, dependency resolve |
| `crates/astro-up-core/src/engine/version_cmp.rs` | Date/custom/semver format comparison |
| `crates/astro-up-core/src/engine/policy.rs` | Update policy enforcement |
| `crates/astro-up-core/src/engine/history.rs` | Operation history (SQLite) |
| `crates/astro-up-core/src/engine/lock.rs` | Global lock file |
| `crates/astro-up-core/src/engine/process.rs` | Running-process detection |

## Dependencies on prior specs

- **005** (catalog): `PackageSummary`, `VersionEntry`, `SqliteCatalogReader`
- **006** (detection): `Detector` trait, `DetectionConfig`
- **010** (download): `Downloader` trait, `DownloadOptions`
- **011** (install): `Installer` trait, `InstallOptions`, `InstallResult`
- **013** (backup): `BackupManager` trait, `BackupResult`

## How it fits together

```
CLI/GUI
  └── UpdateOrchestrator::new(catalog, detector, downloader, installer, backup_mgr, db)
        ├── .plan(request)     → UpdatePlan (for dry-run or confirmation)
        └── .execute(plan)     → OrchestrationResult
              ├── acquire lock
              ├── for each package (topo-sorted):
              │     ├── check process not running
              │     ├── compare versions (format-aware)
              │     ├── download installer
              │     ├── backup config (if configured)
              │     ├── execute installer
              │     ├── verify install (re-detect)
              │     └── log operation to history
              └── release lock
```

## Running tests

```sh
# Unit tests (version parsing, policy logic, dependency resolution)
cargo test -p astro-up-core engine

# Integration tests (full pipeline with mock traits)
cargo test -p astro-up-core --test engine_orchestrator
```

## New dependencies

| Crate | Purpose |
|-------|---------|
| regex | Custom version format parsing |
| fd-lock | File locking + disk space check |
