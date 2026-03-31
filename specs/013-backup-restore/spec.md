# Feature Specification: Backup and Restore

**Feature Branch**: `013-backup-restore`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 012 — backup and restore application configuration files

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatic Backup Before Update (Priority: P1)

Before updating software, the orchestration engine automatically backs up the application's configuration files (defined in the manifest's `[backup]` section). The backup is a timestamped ZIP archive stored locally. If a manifest defines multiple config paths (e.g., NINA profiles in `%APPDATA%`, sequence files in `Documents`), all paths are included in a single archive.

**Why this priority**: Config loss during updates is the #1 user complaint. Automatic backup prevents it.

**Independent Test**: Trigger a backup for NINA, verify the ZIP contains files from all configured paths.

**Acceptance Scenarios**:

1. **Given** NINA manifest defines `config_paths = ["{config_dir}/NINA/Profiles", "{config_dir}/NINA/Settings", "{home_dir}/Documents/N.I.N.A"]`, **When** backup runs, **Then** a single ZIP archive contains files from all three locations
2. **Given** a config path doesn't exist (e.g., no Documents/N.I.N.A yet), **When** backup runs, **Then** the missing path is skipped with a warning
3. **Given** backup completes, **When** inspecting the archive, **Then** it contains metadata with package ID, version, timestamp, and the original paths

---

### User Story 2 - Manual Backup (Priority: P2)

A user runs `astro-up backup nina` before making risky config changes (tweaking a complex sequence profile, experimenting with plugin settings). This creates the same backup as the automatic pre-update backup, on demand.

**Why this priority**: Users want a safety net before manual experiments, not just before astro-up-driven updates.

**Independent Test**: Run `backup nina`, verify an archive is created with the same structure as automatic backups.

**Acceptance Scenarios**:

1. **Given** the user runs `astro-up backup nina`, **When** backup completes, **Then** an archive is created at `{data_dir}/astro-up/backups/{package_id}/` (e.g., `backups/nina-app/`)
2. **Given** the package has no `[backup]` section in its manifest, **When** manual backup runs, **Then** an error reports "no backup paths configured for this package"
3. **Given** NINA is currently running and has locked config files, **When** backup runs, **Then** locked files are skipped with a warning listing which files were excluded

---

### User Story 3 - Restore from Backup (Priority: P3)

A user runs `astro-up restore nina` and selects from available backups. The selected backup is extracted to the original paths. Before overwriting, the user sees which files will change.

**Why this priority**: Backup without restore is useless.

**Independent Test**: Backup NINA config, modify a profile, restore, verify the profile is restored.

**Acceptance Scenarios**:

1. **Given** a backup exists for NINA, **When** `restore nina` runs, **Then** available backups are listed sorted by date with version info
2. **Given** the user selects a backup, **When** restoring, **Then** a summary shows files that will be overwritten vs files unchanged vs files that are new
3. **Given** the backup was from version 3.0 and current version is 3.1, **When** restoring, **Then** a warning says "This backup was created with version 3.0, you are running 3.1"
4. **Given** the user confirms, **When** restore proceeds, **Then** files are extracted to their original paths

---

### User Story 4 - Selective Restore (Priority: P4)

A user only wants to restore their NINA profiles, not all settings. They run `astro-up restore nina --path "Profiles/"` to restore just that directory from the backup.

**Why this priority**: Full restore may overwrite settings the user intentionally changed. Selective restore gives granular control.

**Independent Test**: Backup with profiles + settings, restore only profiles, verify settings are untouched.

**Acceptance Scenarios**:

1. **Given** a backup with Profiles/ and Settings/ directories, **When** restoring with `--path "Profiles/"`, **Then** only Profiles/ is extracted
2. **Given** an invalid `--path` that doesn't exist in the backup, **When** restoring, **Then** an error lists the available paths in the archive

---

### User Story 5 - List and Prune Backups (Priority: P5)

A user runs `astro-up backup list nina` to see all backups. Old backups are automatically pruned after each new backup to stay within the retention limit.

**Why this priority**: Disk space management without losing recovery capability.

**Independent Test**: Create 7 backups with retention of 5, verify only 5 remain after the 7th.

**Acceptance Scenarios**:

1. **Given** 7 backups exist and retention is 5, **When** a new backup is created, **Then** the 2 oldest are deleted
2. **Given** `backup list nina` runs, **Then** backups are shown with date, version, file count, and total size
3. **Given** retention set to 0 (unlimited), **When** pruning runs, **Then** no backups are deleted

### Edge Cases

- Locked files (app running): Skip with warning listing excluded files. The backup is still created — just incomplete. The user can close the app and re-backup.
- Backup path is a symlink: Follow symlinks and archive the target files (not the symlink itself).
- Restore to a path that no longer exists (user moved their Documents folder): Create the target directory.
- Archive corrupted: Report corruption when listing or restoring. Don't auto-delete.
- Backup for a package not in the catalog (removed): Allow backup/restore via the local ledger — the package doesn't need to be in the catalog.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST backup all manifest-defined `[backup].config_paths` into a single timestamped ZIP archive
- **FR-002**: System MUST support multiple config paths per package (different directories, different drives)
- **FR-003**: Backup paths MUST be expanded before reaching the backup module. The caller (orchestration engine or CLI) expands path tokens (`{config_dir}`, `{home_dir}`, etc.) using the config module's `directories`-based path resolution (spec 004). `BackupRequest.config_paths` contains absolute, already-expanded paths.
- **FR-004**: System MUST include `metadata.json` in the archive with: `package_id`, `version`, `created_at`, `paths` (original config_paths), `file_count`, `total_size` (bytes), `excluded_files` (skipped/locked), `file_hashes` (relative_path -> SHA-256 for restore preview)
- **FR-005**: System MUST support manual on-demand backup via CLI and GUI
- **FR-006**: System MUST support automatic backup triggered by the orchestration engine before install/update
- **FR-007**: System MUST restore backup archives to the original file paths
- **FR-008**: System MUST show a file change summary before restore (overwritten / unchanged / new)
- **FR-009**: System MUST warn on version mismatch between backup and currently installed version
- **FR-010**: System MUST support selective restore via `--path` filter
- **FR-011**: System MUST list available backups per package sorted by date with version, file count, size
- **FR-012**: System MUST prune old backups beyond configurable retention count (default: 5) after each new backup
- **FR-013**: System MUST skip locked files during backup with a warning listing excluded files
- **FR-014**: System MUST store backups at `{data_dir}/astro-up/backups/{package_id}/`
- **FR-015**: System MUST name archives `{package_id}_{version}_{YYYYMMDD_HHMMSS}.zip` where `version` is the currently installed version at backup time (not the version being updated to)
- **FR-016**: System MUST preserve relative directory structure within the archive. Each config_path gets a top-level directory named after its last path component (e.g., `Profiles/`, `Settings/`). If two config_paths share the same last component, disambiguate with a numeric suffix (e.g., `Settings/`, `Settings_2/`).
- **FR-017**: System MUST emit events: `BackupStarted` (existing), `BackupProgress { id, files_processed, total_files }` (new), `BackupComplete` (existing), `RestoreStarted { id }` (new), `RestoreComplete { id }` (new). Cross-spec change to spec 003 Event enum.

### Key Entities

- **BackupArchive**: ZIP file containing config files from all configured paths + metadata.json
- **BackupMetadata**: package_id: String, version: Version, created_at: DateTime\<Utc\>, paths: Vec\<PathBuf\> (original config_paths), file_count: u32, total_size: u64 (bytes), excluded_files: Vec\<String\> (locked/skipped). Replaces the minimal `BackupResult` struct in `traits.rs` (cross-spec change to spec 003).
- **FileChangeSummary**: Per-file status for restore preview: overwrite (file differs), unchanged (identical), new (doesn't exist locally)
- **BackupConfig**: `config_paths` list from the software manifest's `[backup]` section. Contains only paths — retention count is a global app config setting (`backup.retention_count`, spec 004), not per-package.

## Clarifications

### Session 2026-03-31

- Q: Should backup module implement its own path token expansion or reuse config system? → A: Reuse config module's `directories`-based path resolution (spec 004). DRY — tokens already handled there.
- Q: Should BackupManager trait be kept minimal or evolved to match spec? → A: Evolve the trait to match spec requirements — add `restore_preview()`, richer return types (`FileChangeSummary`), metadata-aware operations. Cross-spec change to spec 003 trait definition.
- Q: Should backup/restore emit events for progress tracking? → A: Add `BackupProgress { id, files_processed, total_files }`, `RestoreStarted { id }`, `RestoreComplete { id }` event variants. Covers GUI progress for large config directories.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Backup completes in under 10 seconds for typical config sizes (<100MB)
- **SC-002**: Restore recreates all backed-up files at their original paths — byte-identical content, original directory structure preserved. File permissions and timestamps are not guaranteed (ZIP limitations).
- **SC-003**: Pruning correctly retains only the configured number of most recent backups
- **SC-004**: File change summary correctly identifies overwritten vs unchanged files before restore

## Default Values

| Setting | Default | Config Key |
|---------|---------|------------|
| Backup retention | 5 per package | `backup.retention_count` |

## Assumptions

- Only config files are backed up, not the application binaries (those can be re-downloaded)
- Backup runs automatically before install (spec 012) if `[backup]` is defined in the manifest
- ZIP format for portability and built-in compression
- No VSS (Volume Shadow Copy) — skip locked files instead. VSS requires admin and adds OS-specific complexity.
- Backup metadata does NOT validate config content or format — the application handles config migration on startup
- Depends on: spec 003 (types), spec 004 (config for backup retention, path tokens)
