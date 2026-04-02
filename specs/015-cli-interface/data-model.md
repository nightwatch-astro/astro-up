# Data Model: CLI Interface

**Date**: 2026-04-02 | **Spec**: [spec.md](spec.md)

## Key Entities

### OutputMode

Determines how all command output is formatted. Set once at startup, threaded through all commands.

| Variant | Condition | Behavior |
|---------|-----------|----------|
| Interactive | TTY + no `--json` + no `--quiet` | Styled tables, ratatui TUI progress, color |
| Plain | Not TTY or `--quiet` | Plain text, no colors, no TUI |
| Json | `--json` flag | Structured JSON to stdout, no prompts |

### Verbosity

Controls stderr log output level. Independent of file logging (always DEBUG).

| Flag | Level | Notes |
|------|-------|-------|
| (default) | INFO | Normal operation messages |
| `--verbose` | DEBUG | Detailed tracing output |
| `--quiet` | WARN | Errors and warnings only |

### Command Output Types

Each command produces a typed output struct that can be rendered in any OutputMode:

| Command | Output Type | Fields |
|---------|-------------|--------|
| show (all/installed/outdated) | `PackageList` | `Vec<PackageRow>` — name, category, installed_version, latest_version, status |
| show \<package\> | `PackageDetail` | name, version, category, detection_method, backup_count, last_updated, dependencies |
| show backups | `BackupList` | `Vec<BackupRow>` — package, date, version, file_count, size |
| scan | `ScanResult` (from core) | detection results per package |
| search | `SearchResults` | `Vec<PackageRow>` with relevance |
| install/update | `OperationResult` (from engine) | succeeded, failed, cancelled per package |
| backup | `BackupMetadata` (from core) | archive path, file count, size |
| restore | `RestoreResult` | files restored count |
| config show | `Config` (from core) | effective configuration |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (any failure) |
| 2 | User cancelled (Ctrl+C or declined prompt) |

## State Transitions

The CLI is stateless — it reads from core services and exits. No persistent CLI-level state beyond the log file.

### First-Run Bootstrap Flow

```
Start → Check catalog exists?
  No  → Download catalog (with progress) → Check scan cache?
  Yes → Check scan cache?
    No  → Auto-scan (with progress) → Execute command
    Yes → Execute command
```

## Relationships to Core

| CLI Concept | Core Module | Core Type |
|-------------|-------------|-----------|
| show/scan | `detect::Scanner` | `ScanResult`, `PackageDetection` |
| install/update | `engine::Orchestrator` | `UpdatePlan`, `OperationResult` |
| backup/restore | `backup::BackupService` | `BackupMetadata`, `FileChangeSummary` |
| search | `catalog::Catalog` | `Software` |
| config | `config::Config` | `AppConfig` |
