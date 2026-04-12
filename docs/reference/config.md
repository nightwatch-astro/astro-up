# Configuration File Reference

Complete reference for `config.toml`. All settings are also configurable from the **GUI Settings panel** and via **CLI flags**.

See [Configuration Guide](/guide/configuration) for an overview.

## File Location

```
%APPDATA%\nightwatch\astro-up\config.toml
```

## Complete Example

```toml
[ui]
font_size = "medium"
auto_scan_on_launch = true
default_install_scope = "user"
default_install_method = "silent"
auto_check_updates = true
check_interval = "12h"
auto_notify_updates = true

[catalog]
url = "https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db"
cache_ttl = "12h"

[network]
connect_timeout = "10s"
timeout = "30s"
download_speed_limit = 0

[backup_policy]
scheduled_enabled = false
schedule = "weekly"
max_per_package = 5
max_total_size_mb = 0
max_age_days = 0

[notifications]
enabled = true
display_duration = 5
show_errors = true
show_warnings = true
show_update_available = true
show_operation_complete = true

[log]
level = "info"
log_to_file = false

[startup]
start_at_login = false
minimize_to_tray_on_close = true
start_minimized = false

[paths]
download_dir = ""
backup_dir = ""
```

## Field Reference

::: tip GUI Settings
Every field below has a corresponding control in the GUI Settings panel. The table column shows which Settings tab contains it.
:::

### `[ui]`

| Field | Type | Default | GUI tab | Description |
|-------|------|---------|---------|-------------|
| `font_size` | `"small"` \| `"medium"` \| `"large"` | `"medium"` | General | UI font size |
| `auto_scan_on_launch` | bool | `true` | General | Scan for installed software on app start |
| `default_install_scope` | `"user"` \| `"machine"` | `"user"` | General | Default install scope |
| `default_install_method` | `"silent"` \| `"interactive"` | `"silent"` | General | Default installer mode |
| `auto_check_updates` | bool | `true` | General | Periodically check for package updates |
| `check_interval` | duration | `"12h"` | General | How often to check for updates |
| `auto_notify_updates` | bool | `true` | General | Show notification when updates found |

### `[catalog]`

| Field | Type | Default | GUI tab | Description |
|-------|------|---------|---------|-------------|
| `url` | string | GitHub Releases URL | Catalog | Catalog database download URL |
| `cache_ttl` | duration | `"12h"` | Catalog | How long before re-syncing |

### `[network]`

| Field | Type | Default | GUI tab | Description |
|-------|------|---------|---------|-------------|
| `proxy` | string? | none | Network | HTTP proxy URL |
| `connect_timeout` | duration | `"10s"` | Network | TCP connection timeout |
| `timeout` | duration | `"30s"` | Network | Full request timeout |
| `download_speed_limit` | u64 | `0` | Network | Max download speed in bytes/sec (0 = unlimited) |

### `[backup_policy]`

| Field | Type | Default | GUI tab | Description |
|-------|------|---------|---------|-------------|
| `scheduled_enabled` | bool | `false` | Backup | Enable scheduled backups |
| `schedule` | `"daily"` \| `"weekly"` \| `"monthly"` | `"weekly"` | Backup | Backup schedule |
| `max_per_package` | u32 | `5` | Backup | Max backups per package |
| `max_total_size_mb` | u64 | `0` | Backup | Max total backup size in MB (0 = unlimited) |
| `max_age_days` | u32 | `0` | Backup | Delete backups older than N days (0 = never) |

### `[notifications]`

| Field | Type | Default | GUI tab | Description |
|-------|------|---------|---------|-------------|
| `enabled` | bool | `true` | Notifications | Enable desktop notifications |
| `display_duration` | u32 | `5` | Notifications | Seconds before auto-dismiss (0 = never) |
| `show_errors` | bool | `true` | Notifications | Show error notifications |
| `show_warnings` | bool | `true` | Notifications | Show warning notifications |
| `show_update_available` | bool | `true` | Notifications | Show update notifications |
| `show_operation_complete` | bool | `true` | Notifications | Show operation complete notifications |

### `[log]`

| Field | Type | Default | GUI tab | Description |
|-------|------|---------|---------|-------------|
| `level` | log level | `"info"` | Logging | Log level (error, warn, info, debug, trace) |
| `log_to_file` | bool | `false` | Logging | Write logs to file |
| `log_file` | string? | auto | Logging | Log file path (auto-set when enabled) |

### `[startup]`

| Field | Type | Default | GUI tab | Description |
|-------|------|---------|---------|-------------|
| `start_at_login` | bool | `false` | General | Launch at system startup |
| `minimize_to_tray_on_close` | bool | `true` | General | Minimize to tray on close |
| `start_minimized` | bool | `false` | General | Start minimized to tray |

### `[paths]`

| Field | Type | Default | GUI tab | Description |
|-------|------|---------|---------|-------------|
| `download_dir` | string | app data dir | Paths | Downloaded installers directory |
| `backup_dir` | string | app data dir | Paths | Backup archive directory |

## Duration Format

Duration values use [humantime](https://docs.rs/humantime/) format: `"30s"`, `"5m"`, `"1h"`, `"12h"`, `"1day"`, `"7days"`.

## Validation

Configuration is validated on load using [garde](https://github.com/jprochazk/garde). Invalid values produce clear error messages with the field name and constraint that failed.
