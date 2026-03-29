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
