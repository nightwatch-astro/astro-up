# Tasks: Backup and Restore

**Input**: Design documents from `/specs/013-backup-restore/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/backup-service.rs, research.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US5)
- Paths relative to `crates/astro-up-core/`

## Phase 1: Setup

**Purpose**: Add dependencies and wire module structure

- [ ] T001 Add `walkdir = "2"` and `zip = "2"` (if not already present) to `crates/astro-up-core/Cargo.toml` dependencies
- [ ] T002 Create `crates/astro-up-core/src/backup/mod.rs` with module declarations and `BackupService` struct skeleton
- [ ] T003 Wire `pub mod backup;` in `crates/astro-up-core/src/lib.rs`

---

## Phase 2: Foundational (Cross-spec type changes)

**Purpose**: New types, evolved trait, new events. MUST complete before any US work.

- [ ] T004 [P] Create `BackupMetadata`, `FileChangeSummary`, `BackupListEntry`, `BackupRequest`, `RestoreRequest` structs in `crates/astro-up-core/src/backup/types.rs`
- [ ] T005 [P] Add `BackupProgress { id: String, files_processed: u32, total_files: u32 }`, `RestoreStarted { id: String }`, `RestoreComplete { id: String }` event variants to `crates/astro-up-core/src/events.rs`
- [ ] T006 Evolve `BackupManager` trait in `crates/astro-up-core/src/traits.rs`: replace `backup/restore/list/prune` signatures with `BackupRequest/RestoreRequest/BackupMetadata/FileChangeSummary/BackupListEntry` types, add `restore_preview()` method
- [ ] T007 Update snapshot tests for events in `crates/astro-up-core/src/snapshots/`

**Checkpoint**: All shared types updated, `cargo check` passes

---

## Phase 3: User Story 1 - Automatic Backup Before Update (Priority: P1)

**Goal**: Create timestamped ZIP archives of all manifest-defined config_paths before an update. Skip locked files with warning.

**Independent Test**: Create a temp directory tree, run backup, verify ZIP contains all files + metadata.json.

- [ ] T008 [P] [US1] Implement `count_files(paths: &[PathBuf]) -> u32` helper using `walkdir` in `crates/astro-up-core/src/backup/archive.rs` to count total files for progress events
- [ ] T009 [US1] Implement `create_backup(request: &BackupRequest, backup_dir: &Path) -> Result<BackupMetadata, CoreError>` in `crates/astro-up-core/src/backup/archive.rs` that walks each config_path with walkdir, hashes files with SHA-256, writes to ZIP with relative paths, skips locked files (OS error 32), writes metadata.json, emits BackupProgress events
- [ ] T010 [US1] Implement archive naming per FR-015 `{package_id}_{version}_{YYYYMMDD_HHMMSS}.zip` in `crates/astro-up-core/src/backup/archive.rs`
- [ ] T011 [US1] Wire `BackupService::backup()` in `crates/astro-up-core/src/backup/mod.rs` to call `create_backup()`, ensure backup directory exists at `{backup_dir}/{package_id}/`
- [ ] T012 [P] [US1] Write tests for backup creation in `crates/astro-up-core/src/backup/archive.rs` covering: normal multi-path backup, locked file skip, empty directory, metadata.json content verification, file hash verification

**Checkpoint**: Backup creates correct ZIP with metadata, skips locked files

---

## Phase 4: User Story 2 - Manual Backup (Priority: P2)

**Goal**: On-demand backup via CLI/GUI with same archive format. Error if no backup paths configured.

**Independent Test**: Call backup for a package with no [backup] section, verify error.

- [ ] T013 [US2] Add validation in `BackupService::backup()` in `crates/astro-up-core/src/backup/mod.rs` to return error when `config_paths` is empty ("no backup paths configured for this package")
- [ ] T014 [P] [US2] Write test for empty config_paths error in `crates/astro-up-core/src/backup/archive.rs`

**Checkpoint**: Manual backup works, empty paths rejected with clear error

---

## Phase 5: User Story 3 - Restore from Backup (Priority: P3)

**Goal**: Extract backup archive to original paths. Show version mismatch warning.

**Independent Test**: Backup config, modify a file, restore, verify file restored to original content.

- [ ] T015 [US3] Implement `restore(request: &RestoreRequest) -> Result<(), CoreError>` in `crates/astro-up-core/src/backup/archive.rs` that reads metadata.json from archive, extracts files to original paths from metadata.paths, creates missing directories, emits RestoreStarted/RestoreComplete events
- [ ] T016 [P] [US3] Implement `restore_preview(archive_path: &Path) -> Result<FileChangeSummary, CoreError>` in `crates/astro-up-core/src/backup/preview.rs` that reads file_hashes from metadata.json, hashes on-disk files, compares and returns overwritten/unchanged/new/missing lists
- [ ] T017 [US3] Wire `BackupService::restore()` and `BackupService::restore_preview()` in `crates/astro-up-core/src/backup/mod.rs`
- [ ] T018 [P] [US3] Write tests for restore in `crates/astro-up-core/src/backup/archive.rs` covering: full restore, restore to missing directory, version mismatch metadata
- [ ] T019 [P] [US3] Write tests for restore preview in `crates/astro-up-core/src/backup/preview.rs` covering: changed file, unchanged file, new file (in archive not on disk), missing file (on disk not in archive)

**Checkpoint**: Restore works with file change preview

---

## Phase 6: User Story 4 - Selective Restore (Priority: P4)

**Goal**: Restore only a subset of files matching a path filter.

**Independent Test**: Backup with Profiles/ and Settings/, restore only Profiles/, verify Settings/ untouched.

- [ ] T020 [US4] Add `path_filter` support to `restore()` in `crates/astro-up-core/src/backup/archive.rs` that filters archive entries by prefix match against `request.path_filter`
- [ ] T021 [US4] Add error for invalid path_filter (no matching entries in archive) listing available paths in `crates/astro-up-core/src/backup/archive.rs`
- [ ] T022 [P] [US4] Write tests for selective restore in `crates/astro-up-core/src/backup/archive.rs` covering: filter matches subset, filter matches nothing (error with available paths)

**Checkpoint**: Selective restore works with path filter

---

## Phase 7: User Story 5 - List and Prune Backups (Priority: P5)

**Goal**: List available backups per package sorted by date. Auto-prune beyond retention count.

**Independent Test**: Create 7 backups with retention 5, verify only 5 remain.

- [ ] T023 [P] [US5] Implement `list_backups(backup_dir: &Path, package_id: &str) -> Result<Vec<BackupListEntry>, CoreError>` in `crates/astro-up-core/src/backup/prune.rs` that reads archive directory, parses filenames or reads metadata.json, returns sorted by date descending
- [ ] T024 [P] [US5] Implement `prune_backups(backup_dir: &Path, package_id: &str, keep: usize) -> Result<u32, CoreError>` in `crates/astro-up-core/src/backup/prune.rs` that lists backups, deletes oldest beyond retention count, returns number deleted
- [ ] T025 [US5] Wire `BackupService::list()` and `BackupService::prune()` in `crates/astro-up-core/src/backup/mod.rs`, call prune after each backup
- [ ] T026 [P] [US5] Write tests for list and prune in `crates/astro-up-core/src/backup/prune.rs` covering: list sorted order, prune with retention 5 from 7 backups, prune with retention 0 (unlimited), prune with fewer backups than retention

**Checkpoint**: List and prune work, auto-prune after backup

---

## Phase 8: Polish and Cross-Cutting Concerns

**Purpose**: Trait impl, tracing, symlink handling, snapshot updates

- [ ] T027 Implement `BackupManager` trait for `BackupService` in `crates/astro-up-core/src/backup/mod.rs`
- [ ] T028 [P] Add symlink following to walkdir traversal in `crates/astro-up-core/src/backup/archive.rs` per edge case spec (follow symlinks, archive target files)
- [ ] T029 [P] Add tracing spans and events for all backup/restore operations in `crates/astro-up-core/src/backup/mod.rs`
- [ ] T030 Run `cargo fmt` and `cargo clippy -- -D warnings` across workspace, fix any issues
- [ ] T031 Run `cargo test -p astro-up-core` to verify all tests pass, update broken snapshot tests

---

## Dependencies and Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies, start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1, BLOCKS all user stories
- **Phases 3-7 (User Stories)**: All depend on Phase 2
  - US1 (auto backup) must complete first (archive creation)
  - US2 (manual backup) depends on US1 (adds validation only)
  - US3 (restore) depends on US1 (needs archive to restore from)
  - US4 (selective restore) depends on US3 (extends restore)
  - US5 (list + prune) can start after Phase 2 (independent of restore)
- **Phase 8 (Polish)**: After all user stories

### User Story Dependencies

```
Phase 2 (Foundational)
  |-- US1 (Auto Backup) -- US2 (Manual Backup)
  |                     -- US3 (Restore) -- US4 (Selective Restore)
  |-- US5 (List + Prune) [independent after Phase 2]
```

### Parallel Opportunities

- T004, T005: foundational types and events (different files)
- T008, T012: count helper and tests
- T016, T018, T019: restore preview and tests
- T023, T024: list and prune implementations
- T028, T029: symlinks and tracing

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T007)
3. Complete Phase 3: US1 Auto Backup (T008-T012)
4. STOP and VALIDATE: ZIP created with correct structure and metadata

### Incremental Delivery

1. Setup + Foundational: types compile, module wired
2. US1: auto backup creates correct archives
3. US2: validation for manual backup (trivial)
4. US3: restore with preview
5. US4: selective restore (extends US3)
6. US5: list and prune
7. Polish: trait impl, tracing, fmt, clippy

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Fully cross-platform, no cfg(windows) code
- zip crate is synchronous, wrap in spawn_blocking
- Commit after each completed task or logical group
