# Configuration

Astro-Up uses a layered configuration system stored in SQLite. Every setting is configurable from the GUI, CLI, or config file.

## GUI Settings

Open **Settings** from the sidebar. Options are organized by category.

## CLI

```sh
# Show current configuration
astro-up config show

# Initialize config with defaults
astro-up config init
```

## Config Sections

| Section | Key settings |
|---------|-------------|
| **ui** | `theme`, `font_size`, `scan_interval`, `default_install_scope`, `default_install_method` |
| **startup** | `start_at_login`, `start_minimized`, `minimize_to_tray` |
| **catalog** | `url`, `cache_ttl` |
| **network** | `proxy`, `timeouts`, `speed_limit` |
| **notifications** | `enabled`, plus granular per-type toggles |
| **logging** | `level`, `log_to_file`, `max_age_days` |
| **paths** | `download_dir`, `cache_dir`, `portable_apps_dir`, `keep_installers`, `purge_installers_after_days` |

## Paths

Default data location:

```
%APPDATA%\nightwatch\astro-up\
```

Key directories:

| Path | Purpose |
|------|---------|
| Database | SQLite database with catalog, config, and ledger |
| `download_dir` | Downloaded installers (configurable) |
| `cache_dir` | Catalog cache |
| `portable_apps_dir` | Extracted portable applications with Windows shortcuts |

## Notable Settings

- **`portable_apps_dir`** -- where portable apps are extracted; Astro-Up creates a Windows shortcut for each
- **`keep_installers`** / **`purge_installers_after_days`** -- control whether downloaded installers are kept or cleaned up automatically
- **`speed_limit`** -- cap download bandwidth to avoid saturating your connection
- **`scan_interval`** -- how often the GUI checks for installed software automatically

For the complete field reference, see [Configuration File Reference](/reference/config).
