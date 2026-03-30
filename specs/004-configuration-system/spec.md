# Feature Specification: Configuration System

**Feature Branch**: `004-configuration-system`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 003 — layered configuration with defaults, persistent storage, and CLI overrides

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Default Configuration (Priority: P1)

A user installs astro-up for the first time and launches it. Without changing any settings, the application works with sensible defaults: catalog fetched from the official URL, cache stored in the platform-standard cache directory, downloads saved to the platform-standard download directory. The user never needs to touch configuration unless they want to customize behavior.

**Why this priority**: First-run experience must work out of the box. Every other feature depends on configuration being available with defaults.

**Independent Test**: Launch astro-up with no prior config changes. Verify it resolves all paths correctly, connects to the default catalog source, and operates normally.

**Acceptance Scenarios**:

1. **Given** no config changes have been made, **When** the application starts, **Then** all configuration values are populated with documented defaults
2. **Given** no config changes have been made, **When** the application resolves path defaults, **Then** they expand to platform-appropriate directories (e.g., `%APPDATA%\astro-up\data` on Windows)
3. **Given** no SQLite database exists yet, **When** the application starts, **Then** it operates with compiled defaults without error

---

### User Story 2 - Persistent Config Changes (Priority: P2)

A user changes settings via the CLI (`astro-up config set network.timeout 60s`) or the GUI settings page. The change persists in SQLite and survives application restarts. Both CLI and GUI read the same stored values.

**Why this priority**: Persistent settings are the primary customization mechanism. Users expect changes to survive restarts.

**Independent Test**: Run `astro-up config set network.timeout 60s`, restart the application, verify `config get network.timeout` returns `60s`.

**Acceptance Scenarios**:

1. **Given** a user runs `config set network.timeout 60s`, **When** the application restarts, **Then** `config get network.timeout` returns `60s`
2. **Given** a user runs `config set network.connect_timeout 15s`, **When** the application restarts, **Then** `config get network.connect_timeout` returns `15s`
3. **Given** a user runs `config set` with an invalid value (e.g., `network.timeout -5s`), **When** validation runs, **Then** it reports a clear error and does NOT persist the invalid value
4. **Given** a user runs `config list`, **When** output is displayed, **Then** all settings are shown with their current effective values (default or overridden)
5. **Given** a user runs `config reset network.timeout`, **When** the application reads that setting, **Then** it returns the compiled default

---

### User Story 3 - CLI Flag Overrides (Priority: P3)

A user passes `--verbose` to override the logging level for a single invocation. CLI flags take the highest precedence but are NOT persisted — they only apply to the current run.

**Why this priority**: CLI flags are the standard way to temporarily override settings for a single invocation without modifying stored config.

**Independent Test**: Run `astro-up --verbose check` with config `logging.level` set to `warn` in SQLite. Verify logging is at debug level for this invocation only. Re-run without `--verbose`, verify logging is back to `warn`.

**Acceptance Scenarios**:

1. **Given** `logging.level` is `warn` in SQLite, **When** the user runs with `--verbose`, **Then** logging is at debug level for that invocation only
2. **Given** the user passes `--dry-run`, **When** any write operation would occur, **Then** the operation is skipped with a log message
3. **Given** a CLI flag overrides a value, **When** the application reads config, **Then** the flag value wins over both SQLite and defaults

### Edge Cases

- What happens when the SQLite database doesn't exist yet? The application operates with compiled defaults. The database is auto-created on the first `config set` operation.
- What happens when the SQLite database is corrupt? The application logs a warning, renames the corrupt file to `*.corrupt`, creates a fresh database, and operates with defaults.
- What happens when `config set` receives an unknown key? It reports an error listing the valid keys. No partial write occurs.
- What happens when `config reset` is called on a key that was never set? No-op — the setting already uses its default. No error.
- What happens when `config get` is called on a valid key that was never set? Returns the compiled default value.
- What happens when `config list` is called with no stored settings? Returns all settings with their default values.
- What happens when `config set` receives a value of the wrong type (e.g., `config set updates.auto_check 42`)? Validation error with expected type.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST load configuration in this precedence order (highest to lowest): CLI flags → SQLite stored settings → compiled defaults
- **FR-005**: System MUST validate all configuration values before persisting. Validation errors from garde use the format `"field.path: message"` (e.g., `"network.timeout: duration must be positive"`). Type mismatches are caught during parsing and reported with the key, expected type, and actual value.
- **FR-006**: System MUST provide these configuration sections: catalog (source URL, cache TTL, offline flag), paths (download, cache, data directories), network (proxy, timeout, user agent), updates (check interval, auto-check enabled), logging (level, log-to-file flag, log file path), telemetry (opt-in flag). Downstream specs will add fields to these sections (e.g., `network.download_speed_limit` from spec 010, `backup.retention_count` from spec 013, `ui.*` from spec 016) — the config system MUST support adding new sections and fields without breaking existing stored settings.
- **FR-009**: System MUST operate with the defaults documented in the Default Values table when no settings have been stored and no CLI flags are provided
- **FR-011**: System MUST support boolean, integer, string, duration, and path types in config values. Duration input for `config set` MUST accept human-readable strings (e.g., `"24h"`, `"30s"`, `"500ms"`) via `humantime::parse_duration`.
- **FR-013**: System MUST persist config changes to SQLite via `config_set(key, value)`. Changes MUST survive application restarts. The SQLite database is the same file used by catalog and ledger (single database per app instance).
- **FR-014**: System MUST provide these core API functions: `config_get(store, config, key) -> Result<String>` (returns effective value: stored or default), `config_set(store, config, key, value) -> Result<()>` (validates then persists), `config_list(config, stored) -> Vec<(String, String, bool)>` (all settings with effective values and override flag), `config_reset(store, key) -> Result<()>` (removes stored override, reverts to default — no-op if key was never set, accepts any key without validation since reset on unknown key is harmless).
- **FR-015**: System MUST auto-create the SQLite database and config table on the first `config_set` call. If the database already exists (from catalog/ledger), only create the config table.
- **FR-016**: Both CLI and GUI MUST use the same core config API (Constitution Principle IV — Thin Tauri Boundary). The config module receives a database path; it does not resolve platform directories itself.

### Key Entities

- **AppConfig**: Top-level configuration struct containing all sections. Validatable via garde. Used as the schema/registry — both CLI and GUI enumerate fields from this struct.
- **CatalogConfig**: `url` (source URL), `cache_ttl` (hard expiry — re-fetch after TTL, no stale-while-revalidate). (`offline` removed by spec 005 — if offline, can't download software anyway.)
- **PathsConfig**: `download_dir`, `cache_dir`, `data_dir`, `keep_installers` (bool), `purge_installers_after_days` (u32). (Last two added by spec 010.)
- **NetworkConfig**: `proxy` (URL with optional embedded credentials), `connect_timeout`, `timeout`, `user_agent`, `download_speed_limit` (u64 bytes/sec). (`connect_timeout` and `download_speed_limit` added by spec 010.)
- **UpdateConfig**: `auto_check` (enabled flag), `check_interval` (how often to check for astro-up self-updates — distinct from catalog `cache_ttl`)
- **LogConfig**: `level` (error/warn/info/debug/trace), `log_to_file`, `log_file`
- **TelemetryConfig**: `enabled` (opt-in flag)
- **ConfigStore**: Wraps a rusqlite connection. Provides `get`, `set`, `list`, `reset` operations on the `config_settings` table.

Note: The catalog signature verification public key is hardcoded, not configurable.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Application starts and operates correctly with zero stored config (no prior `config set` calls, no database)
- **SC-003**: Configuration validation catches all invalid values and reports actionable error messages. Target: complete config load + validate pipeline under 100ms (aspirational — validated empirically)
- **SC-004**: `config list` shows all settings with their effective values (stored or default)
- **SC-006**: Layering precedence test passes: set the same field via SQLite and CLI flag, assert CLI wins. Remove CLI flag, assert SQLite wins. Reset SQLite, assert default wins.
- **SC-007**: Persistence test passes: `config set network.timeout 60s` persists, `config get network.timeout` returns `60s` after simulated restart, `config reset network.timeout` reverts to default `30s`

## Default Values

| Section | Key | Default |
|---------|-----|---------|
| catalog | url | `https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db` |
| catalog | cache_ttl | 24h |
| paths | download_dir | (platform cache dir)/downloads |
| paths | cache_dir | (platform cache dir) |
| paths | data_dir | (platform data dir) |
| paths | keep_installers | true |
| paths | purge_installers_after_days | 30 (0 = disabled) |
| network | proxy | none |
| network | connect_timeout | 10s |
| network | timeout | 30s |
| network | user_agent | astro-up/{version} |
| network | download_speed_limit | 0 (unlimited, bytes/sec) |
| updates | auto_check | true |
| updates | check_interval | 24h |
| logging | level | info |
| logging | log_to_file | false |
| logging | log_file | (platform data dir)/astro-up.log |
| telemetry | enabled | false |

Note: Platform directories are resolved at app startup via `directories::ProjectDirs` and passed to the config module as concrete paths. The config module itself has no platform-awareness.

## Assumptions

- The application runs on Windows as the primary platform, with macOS/Linux used for development and CI
- No secrets stored in config for v1 — no GitHub token, no keychain integration. Proxy auth credentials are embedded in the proxy URL.
- Telemetry is opt-in by default (disabled unless explicitly enabled)
- Config is loaded once at startup, immutable for the duration of that run. `config set` persists for next run.
- The SQLite database path is provided by the caller (CLI main.rs or Tauri lib.rs), resolved via `directories::ProjectDirs` at app init — not a config-module concern.

## Dependencies

- **Spec 020** (manifest modernization): The default catalog URL (`catalog.db` on GitHub Releases) depends on the manifest repo publishing this artifact.
- **Specs 010, 013, 016**: Will add config fields to sections defined here. The config system supports this via `#[serde(default)]` on structs. When those specs are implemented, add fields to `AppConfig` sub-structs — `known_keys()` auto-discovers them.
- **Specs 015, 016, 017**: CLI and GUI both consume the config API defined here. NOTE: these specs were written against the original TOML-based design and reference `config init`, `config show`, `--config <path>`, and "save to TOML file". They MUST be updated to use `config get/set/list/reset` when implemented.
- **Path token expansion**: Dropped from this spec during the SQLite pivot. Specs 006 and 013 depend on expanding `{config_dir}`, `{program_dir}`, etc. in manifest paths. This responsibility should move to a shared utility in `astro-up-core` (not config-specific — it's a manifest/detection concern). To be resolved when spec 006 is planned.

## Iterations

### Iteration 2026-03-30: Pivot to SQLite-backed config

**Change**: Switch from TOML file + env var layering (4 layers, config-rs) to SQLite-backed persistence with CLI API (3 layers: defaults → SQLite → CLI flags). Drop config-rs, env vars, TOML file, token expansion. Add config get/set/list/reset API.
**Scope**: Pivot
**Artifacts updated**: spec.md, plan.md, tasks.md, data-model.md, research.md, quickstart.md, decisions.md
**Tasks added**: T001-T024 (new task set)
**Tasks removed**: T001-T036 (entire previous task set replaced)
