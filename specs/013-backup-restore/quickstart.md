# Quickstart: 013 Backup and Restore

## What this feature does

Backs up application configuration files before updates, supports manual backup/restore with selective restore and file change preview, manages retention with automatic pruning.

## Key files

| File | Purpose |
|------|---------|
| `crates/astro-up-core/src/backup/mod.rs` | BackupService facade, BackupManager trait impl |
| `crates/astro-up-core/src/backup/types.rs` | BackupMetadata, FileChangeSummary, BackupListEntry, requests |
| `crates/astro-up-core/src/backup/archive.rs` | ZIP creation (backup) and extraction (restore) |
| `crates/astro-up-core/src/backup/preview.rs` | Restore preview — hash comparison, FileChangeSummary |
| `crates/astro-up-core/src/backup/prune.rs` | Retention-based cleanup of old archives |
| `crates/astro-up-core/src/traits.rs` | Modified BackupManager trait |
| `crates/astro-up-core/src/events.rs` | New BackupProgress, RestoreStarted, RestoreComplete variants |

## How to run

```sh
just check    # Clippy + fmt + tests (cross-platform)
just test     # Full test suite
```

## Cross-platform notes

- Fully cross-platform — no `cfg(windows)` code
- Locked file detection uses standard `File::open()` error handling (OS error 32 on Windows)
- ZIP archive format for portability
- Path tokens expanded via config module (spec 004)

## Dependencies

- `zip = "2"` — ZIP creation and reading (already added by spec 011)
- `walkdir = "2"` — recursive directory traversal (new)
- `sha2 = "0.10"` — file hashing for restore preview (existing)
