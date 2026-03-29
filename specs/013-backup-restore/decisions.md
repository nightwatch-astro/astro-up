# Decisions Report: 013-backup-restore
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions
- **ZIP format**: Portable, compressed, well-supported in Rust (zip crate). No custom format.
- **Metadata in archive**: Include manifest.json inside the ZIP so the backup is self-describing.
- **Skip locked files, don't fail**: NINA may be running during backup. Skip files that can't be read rather than failing the entire backup.
- **Default retention of 5**: Balances disk space with recovery options. Configurable via spec 004.
- **No incremental/differential backups**: Full backup each time. Config sizes are small (<100MB). Incrementals add complexity.

## Questions I Would Have Asked
- Q1: Should we support backup to cloud (S3, OneDrive)? Decision: No — local only. Cloud backup is out of scope for an astronomy tool.
- Q2: Should restore require the same software version? Decision: No — warn but allow. Config formats rarely change between minor versions.

## Clarify-Phase Decisions

### C1: Filename includes version for easy identification
**Decision**: `nina-app_3.1.2_20260329_120000.zip` lets users identify which version a backup is from without opening it. Supports multiple backups of the same version (different timestamps).

### C2: Skip locked files, don't use VSS by default
**Decision**: VSS (Volume Shadow Copy) is complex and requires admin. Most astro app configs are simple files that aren't locked during operation. Skip locked files with a warning. If VSS is needed, it's a future enhancement.

### C3: Prune after backup, not on schedule
**Decision**: Pruning immediately after a new backup ensures the count stays bounded. No background scheduler needed. Simple and predictable.

### C4: Selective restore supported
**Decision**: Users sometimes want to restore just their profile without overwriting all settings. `--file` flag enables this. The backup ZIP is random-accessible.
