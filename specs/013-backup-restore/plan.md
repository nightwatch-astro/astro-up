# Implementation Plan: Backup and Restore

**Branch**: `013-backup-restore` | **Date**: 2026-03-31 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/013-backup-restore/spec.md`

## Summary

Implement the backup and restore subsystem: create timestamped ZIP archives of application config files before updates, support manual backup/restore with selective restore and file change preview, manage retention with automatic pruning. Fully cross-platform.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: zip 2 (existing from spec 011), walkdir 2 (new), sha2 0.10 (existing), chrono (existing), tokio (existing)
**Storage**: Filesystem (ZIP archives in `{data_dir}/astro-up/backups/{package_id}/`)
**Testing**: cargo test, insta (snapshots), tempfile (filesystem fixtures)
**Target Platform**: Cross-platform (Windows primary, macOS/Linux CI)
**Project Type**: Library (astro-up-core crate)
**Performance Goals**: SC-001 backup <10s for <100MB config
**Constraints**: No VSS, skip locked files, ZIP format for portability
**Scale/Scope**: 5 user stories, 17 FRs, 4 SCs

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | New `backup/` module in astro-up-core, no new crate |
| II. Platform Awareness | PASS | Fully cross-platform, locked file handling via std::io errors |
| III. Test-First | PASS | Unit tests for archive creation/extraction, integration tests with tempfile |
| IV. Thin Tauri Boundary | PASS | All logic in core, GUI/CLI are consumers |
| V. Spec-Driven | PASS | Full speckit pipeline |
| VI. Simplicity | PASS | zip + walkdir + sha2, no over-engineering |

No violations.

## Project Structure

### Documentation

```text
specs/013-backup-restore/
  spec.md, decisions.md, plan.md, research.md
  data-model.md, quickstart.md
  contracts/backup-service.rs
  checklists/requirements.md
  tasks.md
```

### Source Code

```text
crates/astro-up-core/src/
  backup/
    mod.rs            # BackupService facade, BackupManager trait impl
    types.rs          # BackupMetadata, FileChangeSummary, BackupListEntry, requests
    archive.rs        # ZIP creation (backup) and extraction (restore)
    preview.rs        # Restore preview with hash comparison
    prune.rs          # Retention-based cleanup
  traits.rs           # Modified: BackupManager trait evolved
  events.rs           # Modified: add BackupProgress, RestoreStarted, RestoreComplete
  lib.rs              # Modified: add pub mod backup
```

## Implementation Phases

### Phase A: Foundation (types + dependencies)

- Add `walkdir = "2"` dependency (zip already present from 011 or add if needed)
- Create `backup/types.rs` with BackupMetadata, FileChangeSummary, BackupListEntry, BackupRequest, RestoreRequest
- Add BackupProgress, RestoreStarted, RestoreComplete event variants
- Evolve BackupManager trait in traits.rs
- Wire `pub mod backup` in lib.rs

### Phase B: Archive creation (backup)

- Create `backup/archive.rs` with `create_backup()`:
  - Walk each config_path with `walkdir`
  - Hash each file with SHA-256 during archiving
  - Skip locked files (OS error 32) with warning
  - Follow symlinks
  - Write metadata.json with BackupMetadata
  - Store files with relative paths per config_path root
  - Emit BackupProgress events
- Name archive per FR-015: `{package_id}_{version}_{YYYYMMDD_HHMMSS}.zip`
- Store in `{data_dir}/astro-up/backups/{package_id}/`

### Phase C: Restore

- Add `restore()` to `backup/archive.rs`:
  - Read metadata.json from archive
  - Extract files to original paths (from metadata.paths)
  - Create missing directories
  - Selective restore via path_filter (FR-010)
  - Emit RestoreStarted/RestoreComplete events
- Add `backup/preview.rs` with `restore_preview()`:
  - Read metadata.json file_hashes
  - Hash on-disk files, compare
  - Return FileChangeSummary (overwritten/unchanged/new/missing)

### Phase D: List + prune

- Add `backup/prune.rs`:
  - List archives in package backup directory
  - Parse filename for metadata (or read metadata.json)
  - Sort by date descending
  - Delete oldest beyond retention count
- Implement `list()` returning BackupListEntry sorted by date

### Phase E: Orchestration + polish

- Implement BackupManager trait for BackupService
- Wire BackupService constructor with backup_dir from config
- Tracing spans for all operations
- Update snapshot tests

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Archive format | ZIP (deflate) | Portable, built-in compression, reuses spec 011 dependency |
| Directory traversal | walkdir | Lightweight, blessed.rs, sufficient for explicit paths |
| File comparison | SHA-256 hashes in metadata.json | Enables preview without re-reading ZIP, sha2 already a dep |
| Locked files | Skip + warn via std::io error | No VSS complexity, per constitution principle VI |
| Path tokens | Reuse config module (spec 004) | DRY, already handles all platform path tokens |
| Trait evolution | Evolve BackupManager with richer types | Avoids mismatch (lesson from spec 011 Installer trait) |

## Cross-Spec Impact

| Spec | Change | Compatibility |
|------|--------|---------------|
| 003 (types) | BackupManager trait: richer signatures, new return types | Breaking, no downstream consumers yet |
| 003 (types) | Replace BackupResult/BackupEntry with BackupMetadata/BackupListEntry | Breaking, no consumers |
| 003 (types) | Event: add BackupProgress, RestoreStarted, RestoreComplete | Additive |
| 004 (config) | Consumes path token expansion | Read-only dependency |
| 012 (orchestration) | Consumes BackupManager for pre-install backup | Forward dependency |
