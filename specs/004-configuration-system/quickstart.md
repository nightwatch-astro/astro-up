# Quickstart: Configuration System

## Usage

### Default (no config changes)

```sh
# Just works — all compiled defaults apply
astro-up check
```

### View current settings

```sh
# List all settings with effective values
astro-up config list

# Get a specific setting
astro-up config get network.timeout
# → 30s (default)
```

### Change a setting

```sh
# Set a value (persists in SQLite)
astro-up config set network.timeout 60s
astro-up config set catalog.offline true
astro-up config set logging.level debug

# Verify
astro-up config get network.timeout
# → 60s

# List shows which values are overridden
astro-up config list
# catalog.url        = https://github.com/...  (default)
# catalog.cache_ttl  = 24h                     (default)
# catalog.offline    = true                    *
# network.timeout    = 60s                     *
# logging.level      = debug                   *
# ...
```

### Reset a setting

```sh
# Revert to compiled default
astro-up config reset network.timeout
astro-up config get network.timeout
# → 30s (default)

# Reset all settings
astro-up config reset --all
```

### CLI flag overrides (single invocation)

```sh
# Override for one run only — not persisted
astro-up --verbose check
astro-up --dry-run update
```

## Precedence

CLI flags > SQLite stored settings > compiled defaults

## Developer Usage

```rust
use astro_up_core::config::{AppConfig, load_config};

// Load with defaults only (empty database)
let config = load_config(db_path, &[])?;

// Load with CLI overrides
let config = load_config(db_path, &[("logging.level", "debug")])?;

// Access fields
println!("Catalog URL: {}", config.catalog.url);
println!("Timeout: {:?}", config.network.timeout);
```

### Config API

```rust
use astro_up_core::config::{ConfigStore, config_get, config_set, config_list, config_reset};

let store = ConfigStore::new(conn)?;

// Set (validates before persisting)
config_set(&store, &current_config, "network.timeout", "60s")?;

// Get (returns effective value — stored or default)
let val = config_get(&store, &current_config, "network.timeout")?;
assert_eq!(val, "60s");

// List (all settings with defaults/override flag)
let settings = config_list(&current_config, &store.list()?);

// Reset (reverts to default)
config_reset(&store, "network.timeout")?;
```

## Testing

```rust
use tempfile::NamedTempFile;

let db = NamedTempFile::new()?;
let config = load_config(db.path(), &[])?;
// All defaults, isolated test database
```
