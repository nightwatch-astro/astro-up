# Research: Configuration System

## R2: garde for validation

**Decision**: `garde` v0.22 with `derive` + `url` features
**Rationale**: Cleaner derive syntax than `validator`, automatic nested validation with `#[garde(dive)]`, custom validators for Duration constraints. Works naturally with serde structs.
**Alternatives**: validator crate (older, more verbose)

**Key patterns**:
- `#[garde(dive)]` on sub-config fields for nested validation
- `#[garde(url)]` for proxy/catalog URL fields
- Custom validator for positive Duration (garde has no built-in Duration support)
- `#[garde(allow_unvalidated)]` to skip annotation on fields that don't need validation

## R8: rusqlite config storage

**Decision**: Key-value table in existing SQLite database
**Rationale**: rusqlite is already in the workspace (catalog, ledger). Config is just another table. No new dependencies. GUI and CLI share the same database file. No file format parsing (TOML), no comment preservation, no token expansion.
**Alternatives**: TOML file (dropped — comments lost on GUI write, token expansion complexity, env var layering unnecessary for desktop app), separate config database (unnecessary — one file is simpler)

**Schema**:
```sql
CREATE TABLE IF NOT EXISTS config_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Operations**:
```rust
// Set
conn.execute("INSERT OR REPLACE INTO config_settings (key, value) VALUES (?1, ?2)", params![key, value])?;

// Get
conn.query_row("SELECT value FROM config_settings WHERE key = ?1", params![key], |row| row.get(0)).optional()?;

// List
let mut stmt = conn.prepare("SELECT key, value FROM config_settings ORDER BY key")?;
stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?.collect()

// Reset
conn.execute("DELETE FROM config_settings WHERE key = ?1", params![key])?;
```

## R9: Duration handling

**Decision**: `humantime::parse_duration` for parsing, store as humantime string in SQLite
**Rationale**: humantime is already a transitive dependency. Human-readable duration strings (`"24h"`, `"30s"`) are user-friendly for `config set` input and readable in the database. No humantime-serde needed — conversion is explicit at the API boundary (parse on `config set`, format on `config get`).
**Format**: `humantime::format_duration(duration).to_string()` for display/storage, `humantime::parse_duration(input)` for parsing.

## R10: Platform directory resolution

**Decision**: Resolved at app init, not in config module
**Rationale**: The config module receives a `db_path: &Path` and resolved platform paths as `AppConfig` defaults. This keeps the config module testable (pass a tempfile path in tests) and platform-agnostic. App startup code in CLI `main.rs` or Tauri `lib.rs` calls `directories::ProjectDirs::from("", "", "astro-up")` once.
**Impact**: `directories` crate stays in the workspace but is NOT a dependency of the config module.
