# Data Model: Portable Apps Folder

## Config: `PathsConfig`

| Field | Type | Default | Storage |
|-------|------|---------|---------|
| `portable_apps_dir` | `PathBuf` | `{data_dir}/../apps/` | `config_settings` table, key `paths.portable_apps_dir` |

Existing fields (`download_dir`, `cache_dir`, `data_dir`, `keep_installers`, `purge_installers_after_days`) unchanged.

## Install Pipeline: `InstallRequest`

No new fields. Existing `install_dir: Option<PathBuf>` is populated by the GUI command layer:

| Package Method | `install_dir` value |
|---------------|-------------------|
| `DownloadOnly` | `Some({portable_apps_dir}/{package-id})` |
| `Portable` | `Some({portable_apps_dir}/{package-id})` |
| All others | `None` (unchanged — installer decides) |

## Ledger: `LedgerEntry`

Existing `install_path: Option<PathBuf>` is populated after successful portable install:

| Scenario | `install_path` value |
|----------|---------------------|
| Before this feature (download-only) | Download parent dir or `None` |
| After this feature (download-only) | `{portable_apps_dir}/{package-id}` |
| Portable | `{portable_apps_dir}/{package-id}` |

## Frontend: `AppConfig`

```typescript
interface PathsConfig {
  download_dir: string;
  cache_dir: string;
  data_dir: string;
  portable_apps_dir: string;  // NEW
  keep_installers: boolean;
  purge_installers_after_days: number;
}
```

## State Transitions

```
DownloadOnly package lifecycle:
  NotInstalled → [Install clicked] → Downloading → Copying/Extracting → Installed
                                                                         └─ Ledger entry with install_path
  Installed → [Update clicked] → Downloading → Replacing in-place → Installed (new version)
  Installed → [Reinstall clicked] → Downloading → Replacing in-place → Installed (same version)
```
