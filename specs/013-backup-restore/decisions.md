# Decisions Report: 013-backup-restore

**Created**: 2026-03-30
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: ZIP format, no custom archive
**Choice**: Standard ZIP. Portable, compressed, random-accessible for selective restore.

### D2: Skip locked files, no VSS
**Choice**: Skip with warning listing excluded files. No Volume Shadow Copy.
**Reasoning**: VSS requires admin. Most astro configs aren't locked during normal operation.

### D3: Prune after each new backup
**Choice**: Delete oldest if over retention limit immediately after creating a new backup.

### D4: Metadata in archive (self-describing)
**Choice**: `metadata.json` inside the ZIP with package_id, version, timestamp, paths, file count, excluded files.

## Clarify-Phase Decisions (Interactive)

### C1: Manual backup supported
**Finding**: User confirmed — safety net before risky config experiments.
**Decision**: `astro-up backup nina` uses the same logic as pre-update automatic backup. Same format, same storage, same pruning.

### C2: Multiple config locations per package
**Finding**: User noted NINA stores things in AppData, Documents, plugin dirs.
**Decision**: `[backup].config_paths` is a list. All paths go into a single archive. Directory structure preserved with source path as prefix:
```
metadata.json
config_dir/NINA/Profiles/...
config_dir/NINA/Settings/...
home_dir/Documents/N.I.N.A/...
```

### C3: No config validation — app handles migration
**Finding**: User agreed config validation isn't feasible.
**Decision**: Version mismatch warning + file change summary. No config content parsing.

### C4: File change summary before restore
**Decision**: Show per-file status before overwriting: overwrite (differs), unchanged (identical), new (not on disk). Gives confidence without parsing config content.

### C5: Selective restore via --path
**Decision**: `restore nina --path "Profiles/"` restores only that subtree from the archive.

## Questions I Would Have Asked

### Q1: Should backup include application binaries?
**My decision**: No — config only. Binaries can be re-downloaded. Config is irreplaceable.

### Q2: Should we encrypt backups?
**My decision**: No — config isn't sensitive enough. If the data dir is compromised, backup encryption alone doesn't help.

### Q3: GUI context menu for backup/restore?
**My decision**: Yes — frontend spec (017) should expose "Backup Now" and "Restore..." per package. This spec is the core logic.
