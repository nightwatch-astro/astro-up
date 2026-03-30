# Data Model: Configuration System

## Entity Relationships

```
AppConfig (top-level)
├── CatalogConfig
├── PathsConfig
├── NetworkConfig
├── UpdateConfig
├── LogConfig
└── TelemetryConfig

TokenResolver (runtime helper, not serialized)
└── resolves {config_dir}, {cache_dir}, {data_dir}, {home_dir}
```

## Entities

### AppConfig

Top-level struct. All fields have defaults via `#[serde(default)]`.

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
| url | String | `https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db` | `#[garde(url)]` |
| cache_ttl | Duration | 24h | custom: positive |
| offline | bool | false | none |

### PathsConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| download_dir | PathBuf | `{cache_dir}/downloads` | custom: parent exists or creatable |
| cache_dir | PathBuf | `{cache_dir}` (from directories) | custom: parent exists or creatable |
| data_dir | PathBuf | `{data_dir}` (from directories) | custom: parent exists or creatable |

Note: Tokens map to `ProjectDirs` methods which already include the app name (e.g., `{cache_dir}` → `~/.cache/astro-up` on Linux). Path defaults use token syntax strings pre-expansion (e.g., `"{cache_dir}/downloads"`). Duration and scalar defaults are concrete values (e.g., `Duration::from_secs(86400)`). After token expansion, all paths are absolute.

### NetworkConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| proxy | Option\<String\> | None | `#[garde(url)]` when Some |
| timeout | Duration | 30s | custom: positive |
| user_agent | String | `astro-up/{version}` | `#[garde(length(min = 1))]` |

### UpdateConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| auto_check | bool | true | none |
| check_interval | Duration | 24h | custom: positive, min 1m |

### LogConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| level | LogLevel | info | none (enum) |
| log_to_file | bool | false | none |
| log_file | PathBuf | `{data_dir}/astro-up.log` | custom: parent exists or creatable |

### TelemetryConfig

| Field | Type | Default | Validation |
|-------|------|---------|------------|
| enabled | bool | false | none |

### LogLevel (enum)

Variants: `Error`, `Warn`, `Info`, `Debug`, `Trace`

Derives: `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Display`, `EnumString`

Maps to `tracing::Level` for runtime use.

### TokenResolver (runtime, not serialized)

| Field | Type | Source |
|-------|------|--------|
| config_dir | PathBuf | `ProjectDirs::config_dir()` |
| cache_dir | PathBuf | `ProjectDirs::cache_dir()` |
| data_dir | PathBuf | `ProjectDirs::data_dir()` |
| home_dir | PathBuf | `BaseDirs::home_dir()` |

Methods:
- `new() -> Result<Self>` — resolve all dirs, fail if `$HOME` unresolvable
- `expand(&self, input: &str) -> Result<PathBuf>` — replace tokens, error on unknown tokens
- `expand_config(&self, config: &mut AppConfig) -> Result<()>` — walk all path fields

## Serde Configuration

All config structs derive `Serialize` + `Deserialize` with:
- `#[serde(default)]` on each struct for forward-compatibility
- `#[serde(with = "humantime_serde")]` on Duration fields
- `#[serde(rename_all = "snake_case")]` (matches TOML convention)

## Config Loading Pipeline

```
1. Config::builder()
2.   .set_default(...)          ← compiled defaults from Default impl
3.   .add_source(File)          ← config.toml (optional)
4.   .add_source(Environment)   ← ASTROUP_ env vars
5.   .set_override(...)         ← CLI args
6.   .build()?
7.   .try_deserialize::<AppConfig>()?
8. ─── unknown key detection (parallel TOML parse + diff) ───
9. ─── token expansion (TokenResolver::expand_config) ───
10. ─── validation (AppConfig::validate()) ───
11. → immutable AppConfig for application lifetime
```

Step 8 (unknown keys) runs in parallel with steps 7-10 conceptually — it parses the raw TOML file independently and logs warnings. It does not block the main pipeline.
