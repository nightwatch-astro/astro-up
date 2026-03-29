# Feature Specification: Backup and Restore

**Feature Branch**: `013-backup-restore`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 012 — backup and restore application configuration files

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Backup Before Update (Priority: P1)

Before updating software, the engine automatically backs up the application's configuration files (defined in the manifest's `[backup]` section). The backup is a timestamped ZIP archive stored in the astro-up data directory.

**Why this priority**: Config loss during updates is the #1 user complaint. Automatic backup prevents it.

**Independent Test**: Trigger a backup for NINA, verify the ZIP contains the expected Profiles and Settings directories.

**Acceptance Scenarios**:

1. **Given** NINA manifest defines `config_paths = ["{config_dir}/NINA/Profiles", "{config_dir}/NINA/Settings"]`, **When** backup runs, **Then** a ZIP archive is created with those directories
2. **Given** a config path doesn't exist, **When** backup runs, **Then** the missing path is skipped with a warning (not an error)
3. **Given** backup completes, **When** checking the archive, **Then** it contains a manifest.json with package ID, version, timestamp

---

### User Story 2 - Restore from Backup (Priority: P2)

A user runs `astro-up restore nina-app` and selects from available backups. The selected backup is extracted to the original paths, restoring the configuration to its previous state.

**Why this priority**: Backup without restore is useless. Users need a reliable way to recover.

**Independent Test**: Backup NINA config, modify a profile, restore from backup, verify the profile is restored.

**Acceptance Scenarios**:

1. **Given** a backup exists for NINA, **When** restore runs, **Then** files are extracted to their original paths
2. **Given** multiple backups exist, **When** the user lists them, **Then** backups are shown sorted by date with version info
3. **Given** a restore would overwrite existing files, **When** restoring, **Then** the user is warned before proceeding

---

### User Story 3 - Prune Old Backups (Priority: P3)

The system automatically prunes old backups beyond a configurable retention count (default: 5 per package).

**Why this priority**: Disk space management — without pruning, backups accumulate indefinitely.

**Independent Test**: Create 7 backups for a package, verify only the 5 most recent remain after pruning.

**Acceptance Scenarios**:

1. **Given** 7 backups exist and retention is 5, **When** pruning runs, **Then** the 2 oldest are deleted
2. **Given** retention is set to 0, **When** pruning runs, **Then** no backups are deleted (0 means unlimited)

### Edge Cases

- Backup path contains locked files (e.g., NINA is running): Skip locked files with a warning.
- Backup archive is corrupted: Report corruption when listing or restoring. Don't delete it automatically.
- Restore to a different version than the backup was made from: Warn but allow (config format may have changed).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST backup manifest-defined config paths into timestamped ZIP archives
- **FR-002**: System MUST expand path tokens in backup config paths before archiving
- **FR-003**: System MUST include backup metadata (package_id, version, timestamp) in the archive
- **FR-004**: System MUST restore backup archives to the original file paths
- **FR-005**: System MUST list available backups per package sorted by date
- **FR-006**: System MUST prune old backups beyond configurable retention count (default: 5)
- **FR-007**: System MUST skip missing config paths during backup with a warning
- **FR-008**: System MUST warn before overwriting existing files during restore
- **FR-009**: System MUST store backups in `{data_dir}/astro-up/backups/{package_id}/`

### Key Entities

- **BackupArchive**: ZIP file containing config files + metadata manifest
- **BackupMetadata**: package_id, version, created_at, file_count, total_size
- **BackupConfig**: config_paths list from the software manifest

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Backup completes in under 10 seconds for typical config sizes (<100MB)
- **SC-002**: Restore accurately recreates all backed-up files at their original paths
- **SC-003**: Pruning correctly retains only the configured number of most recent backups

## Assumptions

- Only config files are backed up, not the application itself (that can be re-downloaded)
- Backup runs automatically before install (spec 012) if `[backup]` is defined in the manifest
- ZIP format chosen for portability and built-in compression
- Depends on: spec 003 (types), spec 004 (config for backup paths/retention)
