# Data Model: CLI Command Wiring

**Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)

## Entities

### CliState

Shared state initialized once per CLI invocation. Mirrors the GUI's AppState pattern.

| Field | Type | Source |
|-------|------|--------|
| data_dir | PathBuf | `directories::ProjectDirs` |
| db_path | PathBuf | `{data_dir}/astro-up.db` |
| config | AppConfig | `config::load_config()` |
| catalog_manager | CatalogManager | `CatalogManager::new()` |
| backup_service | BackupService | `BackupService::new()` |

### OutputMode (updated)

| Variant | Trigger | Behavior |
|---------|---------|----------|
| Interactive | TTY + no flags | indicatif progress bars, colored tables |
| Plain | piped stdout | Line-by-line text, no ANSI |
| Quiet | `--quiet` | Zero stdout, errors to stderr |
| Json | `--json` | Structured JSON to stdout |

### ScanResultRow (display)

| Field | Source |
|-------|--------|
| package_id | DetectionResult.package_id |
| name | Catalog lookup |
| version | DetectionResult.version |
| method | DetectionResult.method |
| status | Installed / UpdateAvailable / NotInstalled |

## State Transitions

### Scan → Ledger → Show

```
scan command
  → Scanner.scan()
  → LedgerStore.record_scan_results()  [persist to SQLite]
  → Display results as table/JSON

show installed
  → LedgerStore.list_installed()  [read from SQLite]
  → Display filtered results

show outdated
  → LedgerStore.list_installed()
  → CatalogReader.latest_version() for each
  → Filter where installed < latest
  → Display
```

### Install / Update Pipeline

```
install/update command
  → CatalogManager.ensure_catalog()
  → Orchestrator.plan(request)
  → Display plan, confirm
  → Orchestrator.execute(plan, event_callback, cancel_token)
    → Event channel → indicatif ProgressBar (Interactive) or text (Plain) or silent (Quiet) or JSON stream (Json)
  → Display result summary
```
