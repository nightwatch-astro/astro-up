# Data Model: Configuration System

## Entity Relationships

```
AppConfig (top-level, in-memory schema)
├── CatalogConfig
├── PathsConfig
├── NetworkConfig
├── UpdateConfig
├── LogConfig
└── TelemetryConfig

ConfigStore (SQLite persistence)
└── config_settings table (key-value)
```

## Entities

### AppConfig

Top-level struct. All fields have defaults. Acts as the parameter registry — both CLI and GUI enumerate fields from this struct.

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| catalog | CatalogConfig | (nested defaults) | `#[garde(dive)]` |
| paths | PathsConfig | (nested defaults) | `#[garde(dive)]` |
| network | NetworkConfig | (nested defaults) | `#[garde(dive)]` |
| updates | UpdateConfig | (nested defaults) | `#[garde(dive)]` |
| logging | LogConfig | (nested defaults) | `#[garde(dive)]` |
| telemetry | TelemetryConfig | (nested defaults) | `#[garde(dive)]` |

### CatalogConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| url | String | (see defaults table) | `#[garde(url)]` |
| cache_ttl | Duration | 24h | custom: positive |
| offline | bool | false | none |

### PathsConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| download_dir | PathBuf | (platform cache dir)/downloads | none (resolved at app init) |
| cache_dir | PathBuf | (platform cache dir) | none |
| data_dir | PathBuf | (platform data dir) | none |

Note: Path defaults are resolved by the caller (app init) and passed to the config module. The config module stores/loads paths as strings in SQLite.

### NetworkConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| proxy | Option\<String\> | None | `#[garde(url)]` when Some |
| timeout | Duration | 30s | custom: positive |
| user_agent | String | astro-up/{version} | `#[garde(length(min = 1))]` |

### UpdateConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| auto_check | bool | true | none |
| check_interval | Duration | 24h | custom: positive, min 1m |

### LogConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| level | LogLevel | Info | none (enum) |
| log_to_file | bool | false | none |
| log_file | PathBuf | (platform data dir)/astro-up.log | none |

### TelemetryConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| enabled | bool | false | none |

### LogLevel (enum)

Variants: `Error`, `Warn`, `Info`, `Debug`, `Trace`

Derives: `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Display`, `EnumString`

Maps to `tracing::Level` for runtime use.

### ConfigStore

Wraps a `rusqlite::Connection`. Provides typed access to the `config_settings` table.

| Method | Signature | Description |
|--------|-----------|-------------|
| new | `(conn: Connection) -> Result<Self>` | Creates table if not exists |
| get | `(key: &str) -> Result<Option<String>>` | Read stored value |
| set | `(key: &str, value: &str) -> Result<()>` | Write value |
| list | `() -> Result<Vec<(String, String)>>` | All stored key-value pairs |
| reset | `(key: &str) -> Result<()>` | Delete stored override |
| reset_all | `() -> Result<()>` | Delete all stored overrides |

## SQLite Schema

```sql
CREATE TABLE IF NOT EXISTS config_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Values are stored as strings. Type conversion happens at the application layer:
- Booleans: `"true"` / `"false"`
- Integers: `"42"`
- Durations: `"24h"`, `"30s"` (humantime format)
- Paths: absolute path strings
- Strings: as-is

## Config Loading Pipeline

```
1. AppConfig::default()              ← compiled defaults (with resolved platform paths)
2. ConfigStore::list()               ← read all stored overrides from SQLite
3. Merge stored values over defaults ← string → typed conversion via serde/humantime
4. Merge CLI flags over result       ← highest precedence, not persisted
5. AppConfig::validate()             ← garde validation
6. → immutable AppConfig for application lifetime
```

## Config Set Pipeline

```
1. Validate key exists in AppConfig field set
2. Parse value to target type (humantime for Duration, FromStr for others)
3. Validate the new value (garde on a temporary AppConfig with this change)
4. ConfigStore::set(key, value_string)
5. → persisted, effective on next load
```
