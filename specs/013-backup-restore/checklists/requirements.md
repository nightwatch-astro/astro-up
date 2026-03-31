# Requirements Quality Checklist: Backup and Restore

**Purpose**: Validate completeness, clarity, consistency, and measurability of spec 013 requirements before planning
**Created**: 2026-03-31
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Does the spec define the metadata.json schema (exact fields, types, format)? The Key Entities section lists fields but not their JSON types or structure. [Completeness, Spec §Key Entities]
- [ ] CHK002 - Are requirements defined for what happens when a backup is in progress and a second backup is requested for the same package? [Gap]
- [ ] CHK003 - Is the maximum backup archive size or config directory size specified? SC-001 says <100MB "typical" but no behavior for >100MB. [Completeness, Spec §SC-001]
- [ ] CHK004 - Are requirements defined for how `BackupManager` trait evolution (clarification Q2) maps to the existing trait in `traits.rs`? [Completeness, Spec §Clarifications]
- [ ] CHK005 - Does the spec define which event channel (`broadcast::Sender<Event>`) the backup module receives for emitting events? [Gap]

## Requirement Clarity

- [ ] CHK006 - Is "locked files" defined precisely — is it Windows file locks only, or also POSIX advisory locks? [Clarity, Spec §FR-013]
- [x] CHK007 - Is the archive naming format `{package_id}_{version}_{YYYYMMDD_HHMMSS}.zip` clear about which version — the currently installed version or the version being updated to? [Clarity, Spec §FR-015] → Resolved: FR-015 specifies "currently installed version at backup time".
- [ ] CHK008 - Is "config files" in the Assumptions section defined — does it include databases (SQLite), binary settings files, or only text-based config? [Clarity, Spec §Assumptions]
- [ ] CHK009 - Is the behavior for `restore_selective` with overlapping paths specified (e.g., `--path "Profiles/"` when archive has both `Profiles/` and `Profiles.bak/`)? [Clarity, Spec §FR-010]

## Requirement Consistency

- [x] CHK010 - Is the backup storage path consistent between US2 (`{data_dir}/astro-up/backups/nina/`) and FR-014 (`{data_dir}/astro-up/backups/{package_id}/`)? US2 uses the package name "nina", FR-014 uses `{package_id}`. [Consistency, Spec §US2/§FR-014] → Resolved: US2 updated to use `{package_id}/` with example.
- [x] CHK011 - Are the `BackupResult` fields in `traits.rs` (software_id, timestamp, path, size) consistent with the richer `BackupMetadata` in the spec (package_id, version, created_at, paths, file_count, total_size, excluded_files)? [Consistency, Cross-spec] → Resolved: BackupMetadata now fully typed with all fields; documented as replacing BackupResult (cross-spec change).
- [ ] CHK012 - Is the `BackupConfig` entity (just `config_paths`) consistent with the Default Values table which adds `backup.retention_count` as a config key? [Consistency, Spec §Key Entities/§Default Values]

## Acceptance Criteria Quality

- [ ] CHK013 - Can SC-001 "<10 seconds for typical config sizes (<100MB)" be objectively measured — is "typical" defined? [Measurability, Spec §SC-001]
- [x] CHK014 - Can SC-002 "accurately recreates all backed-up files" be measured — what defines "accurate" (byte-identical? permissions preserved? timestamps?)? [Measurability, Spec §SC-002] → Resolved: SC-002 now specifies byte-identical content + directory structure; permissions/timestamps not guaranteed (ZIP limitations).
- [ ] CHK015 - Can SC-004 "correctly identifies overwritten vs unchanged" be measured — is the comparison method specified (hash? byte-by-byte? timestamp?)? [Measurability, Spec §SC-004]

## Scenario Coverage

- [ ] CHK016 - Are requirements defined for backing up an empty config directory (exists but contains no files)? [Coverage, Edge Case]
- [ ] CHK017 - Are requirements defined for restore when the target disk is full? [Coverage, Exception Flow]
- [ ] CHK018 - Are requirements defined for concurrent backup and restore operations on the same package? [Coverage, Gap]
- [ ] CHK019 - Are requirements defined for what happens when the backup directory itself becomes very large (hundreds of backups from retention=0)? [Coverage, Edge Case]

## Edge Case Coverage

- [ ] CHK020 - Is the behavior specified for config paths that contain symbolic link loops? FR-014 says follow symlinks, but loops could cause infinite recursion. [Edge Case, Spec §Edge Cases]
- [ ] CHK021 - Is the behavior specified for archives created on a different OS or architecture? (e.g., backup on one Windows machine, restore on another with different drive letters) [Edge Case, Gap]
- [ ] CHK022 - Is the behavior specified for config files that change during backup (TOCTOU — file modified between listing and archiving)? [Edge Case, Gap]

## Cross-Spec Dependencies

- [ ] CHK023 - Are the new Event variants (`BackupProgress`, `RestoreStarted`, `RestoreComplete`) documented with their serialization format for TypeScript consumers? [Dependency, Spec §FR-017]
- [ ] CHK024 - Is the `BackupManager` trait evolution (adding `restore_selective`, `FileChangeSummary`) documented as a breaking change to spec 003? [Dependency, Spec §Clarifications]
- [ ] CHK025 - Is the integration point with spec 012 (orchestration triggers automatic backup) specified — what parameters does 012 pass? [Dependency, Spec §FR-006]

## Notes

- Check items off as completed: `[x]`
- Items referencing `[Gap]` indicate missing requirements that should be added
- Items referencing `[Cross-spec]` require coordination with other specs
