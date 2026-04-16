# Configuration Reference

## Storage

Configuration uses a SQLite dot-path key-value store in the application database. Not a TOML file.

| Item | Value |
|------|-------|
| Database | `astro-up.db` |
| Table | `config_settings` (key TEXT, value TEXT, updated_at TEXT) |
| Location | `%APPDATA%\nightwatch\astro-up\data\` |
| Access | `astro-up config show` / `astro-up config init` / GUI Settings |
| Validation | [garde](https://github.com/jprochazk/garde) on load |
| Duration format | [humantime](https://docs.rs/humantime/) -- `"30s"`, `"5m"`, `"1h"`, `"24h"`, `"7days"` |

Keys use dot-path notation (e.g., `ui.theme`, `network.proxy`). Only keys with non-default values are stored; missing keys resolve to their defaults.

## `ui`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `ui.theme` | `system` \| `dark` \| `light` | `system` | Color theme |
| `ui.font_size` | `small` \| `medium` \| `large` | `medium` | UI font size |
| `ui.auto_scan_on_launch` | bool | `false` | Scan for installed software on startup |
| `ui.scan_interval` | `manual` \| `on_startup` \| `hourly` \| `daily` \| `weekly` | `hourly` | Auto-scan frequency |
| `ui.default_install_scope` | `user` \| `machine` | `user` | Default install scope |
| `ui.default_install_method` | `silent` \| `interactive` | `interactive` | Default installer mode |
| `ui.auto_check_updates` | bool | `true` | Periodically check for package updates |
| `ui.check_interval` | duration | `"24h"` | How often to check for updates (min 1m) |
| `ui.auto_notify_updates` | bool | `true` | Show notification when updates are found |
| `ui.survey_threshold` | u32 | `3` | Successful operations before showing feedback survey |

## `startup`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `startup.start_at_login` | bool | `false` | Launch at system startup |
| `startup.start_minimized` | bool | `false` | Start minimized to tray |
| `startup.minimize_to_tray_on_close` | bool | `false` | Minimize to tray on close |

## `catalog`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `catalog.url` | string (URL) | GitHub Releases URL | Catalog database download URL |
| `catalog.cache_ttl` | duration | `"24h"` | Time before re-syncing catalog |

## `network`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `network.proxy` | string? | none | HTTP proxy URL |
| `network.connect_timeout` | duration | `"10s"` | TCP connection timeout |
| `network.timeout` | duration | `"30s"` | Full request timeout |
| `network.user_agent` | string | `astro-up/{version}` | HTTP User-Agent header |
| `network.download_speed_limit` | u64 | `0` | Max download speed bytes/sec (0 = unlimited) |

## `notifications`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `notifications.enabled` | bool | `true` | Enable desktop notifications |
| `notifications.display_duration` | u32 | `5` | Seconds before auto-dismiss |
| `notifications.show_errors` | bool | `true` | Show error notifications |
| `notifications.show_warnings` | bool | `true` | Show warning notifications |
| `notifications.show_update_available` | bool | `true` | Show update available notifications |
| `notifications.show_operation_complete` | bool | `true` | Show operation complete notifications |

## `paths`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `paths.download_dir` | path | app data dir | Downloaded installers directory |
| `paths.cache_dir` | path | app data dir | Cache directory |
| `paths.data_dir` | path | app data dir | Application data directory |
| `paths.portable_apps_dir` | path | app data dir | Portable apps install directory |
| `paths.keep_installers` | bool | `true` | Keep downloaded installers after install |
| `paths.purge_installers_after_days` | u32 | `30` | Delete kept installers after N days |

## `updates`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `updates.auto_check` | bool | `true` | Auto-check for astro-up updates |
| `updates.check_interval` | duration | `"24h"` | Self-update check interval (min 1m) |

## `logging`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `logging.level` | `error` \| `warn` \| `info` \| `debug` \| `trace` | `info` | Log level |
| `logging.log_to_file` | bool | `false` | Write logs to file |
| `logging.log_file` | path | auto | Log file path |
| `logging.max_age_days` | u32 | `365` | Delete log files older than N days (0 = never) |
