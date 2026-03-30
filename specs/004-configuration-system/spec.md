# Feature Specification: Configuration System

**Feature Branch**: `004-configuration-system`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 003 — layered configuration loading with defaults, TOML file, and environment variables

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Default Configuration (Priority: P1)

A user installs astro-up for the first time and launches it. Without creating any config file, the application works with sensible defaults: catalog fetched from the official URL, cache stored in the platform-standard cache directory, downloads saved to the platform-standard download directory. The user never needs to touch a config file unless they want to customize behavior.

**Why this priority**: First-run experience must work out of the box. Every other feature depends on configuration being available with defaults.

**Independent Test**: Launch astro-up with no config file present. Verify it resolves all paths correctly, connects to the default catalog source, and operates normally.

**Acceptance Scenarios**:

1. **Given** no config file exists, **When** the application starts, **Then** all configuration values are populated with documented defaults
2. **Given** no config file exists, **When** the application resolves `{config_dir}`, **Then** it expands to the platform-appropriate configuration directory (e.g., `%APPDATA%/astro-up` on Windows)
3. **Given** no config file exists, **When** the application resolves `{cache_dir}`, **Then** it expands to the platform-appropriate cache directory

---

### User Story 2 - TOML Config File (Priority: P2)

A power user creates a `config.toml` file at `{config_dir}/astro-up/config.toml` to customize behavior. They change the download directory and set a proxy. The TOML values override the defaults but can themselves be overridden by environment variables.

**Why this priority**: TOML config is the primary customization mechanism for persistent settings. Power users expect file-based configuration.

**Independent Test**: Create a config.toml with custom values, launch the application, verify the custom values are used.

**Acceptance Scenarios**:

1. **Given** a config.toml with `download_dir = "D:\\Astro\\Downloads"`, **When** the application loads config, **Then** the download directory resolves to `D:\Astro\Downloads`
2. **Given** a config.toml with `proxy = "http://proxy.local:8080"`, **When** the application makes HTTP requests, **Then** traffic routes through the specified proxy
3. **Given** a config.toml with an invalid value (e.g., negative update interval), **When** the application loads config, **Then** it reports a clear validation error with the field name and constraint

---

### User Story 3 - Environment Variable Overrides (Priority: P3)

A CI/CD system or Docker container runs astro-up with configuration injected via environment variables. The `ASTROUP_` prefix maps to config fields: `ASTROUP_CATALOG__URL` sets the catalog source URL (double underscore for nesting). Environment variables take highest precedence after CLI arguments.

**Why this priority**: Environment variables enable headless/automated operation and configuration injection without config files on disk.

**Independent Test**: Set `ASTROUP_CATALOG__URL=https://custom.example.com/catalog.db`, launch the application, verify the custom catalog URL is used.

**Acceptance Scenarios**:

1. **Given** `ASTROUP_CATALOG__URL=https://custom.example.com/catalog.db` is set, **When** the application loads config, **Then** the catalog URL points to the custom location regardless of what the TOML file says
2. **Given** `ASTROUP_LOGGING__LEVEL=debug` is set, **When** the application starts, **Then** logging is at debug level
3. **Given** an env var with an invalid name like `ASTROUP_NONEXISTENT_FIELD`, **When** the application loads config, **Then** the unknown variable is silently ignored

---

### User Story 4 - CLI Argument Overrides (Priority: P4)

A user passes `--config /path/to/custom.toml` to use an alternate config file, or `--verbose` to override the logging level. CLI arguments take the highest precedence in the layering hierarchy.

**Why this priority**: CLI flags are the most common way to temporarily override settings for a single invocation.

**Independent Test**: Run `astro-up check --verbose`, verify logging is at debug level even if the config file sets it to `info`.

**Acceptance Scenarios**:

1. **Given** a config.toml with `log_level = "warn"`, **When** the user runs with `--verbose`, **Then** logging is at debug level
2. **Given** the user passes `--config /tmp/test.toml`, **When** the application starts, **Then** it loads the specified file instead of the default location
3. **Given** the user passes `--dry-run`, **When** any write operation would occur, **Then** the operation is skipped with a log message

### Edge Cases

- What happens when the config file has a syntax error? The application reports the TOML parse error with line/column and exits with a non-zero code.
- What happens when a path token like `{program_dir}` cannot be resolved? The application reports the unresolvable token and the field it was used in. `{program_dir}` is NOT a global token — it's per-package (resolved from detection, not config). Config only uses `{config_dir}`, `{cache_dir}`, `{data_dir}`, `{home_dir}`.
- What happens when the config directory doesn't exist? The application creates it on first write (e.g., when saving defaults).
- What happens when environment variables conflict with TOML config? Environment variables always win (documented precedence).
- What happens when the config format changes between versions? Config is forward-compatible: unknown keys are warned but not rejected. Missing keys use defaults. No explicit version field needed — serde defaults handle migration.
- What happens when the config.toml is empty (0 bytes)? Valid TOML with no keys — treated as no overrides, all defaults apply.
- What happens when `{cache_dir}` or `{data_dir}` cannot be resolved (e.g., `$HOME` unset on Linux CI)? The application reports the unresolvable token, the field it was used in, and exits with a non-zero code.
- What happens when paths resolve to locations with non-ASCII characters or spaces (e.g., `C:\Users\José\AppData`)? The platform directories crate handles this natively; no special treatment needed.
- What happens when `config show` is run with no config file? It shows the effective configuration (compiled defaults with any env var overrides applied).
- What happens when `config init` is run and a config file already exists? It exits with an error message pointing to the existing file. Use `--force` to overwrite.
- What happens when `--config /path/to/custom.toml` is combined with environment variables? Env vars still apply on top of the custom file — the layering precedence is always CLI → env → file → defaults, regardless of which file is loaded.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST load configuration in this precedence order (highest to lowest): CLI arguments → environment variables → TOML config file → compiled defaults. Implementation note: CLI overrides are applied via `.set_override("key", value)` on the `ConfigBuilder`, which is the highest-precedence layer.
- **FR-002**: System MUST use the `ASTROUP_` prefix for environment variable mapping, with double underscore (`__`) as the ONLY nesting delimiter (e.g., `ASTROUP_CATALOG__URL` maps to `catalog.url`). Single underscores are literal (e.g., `ASTROUP_UPDATES__CHECK_INTERVAL` maps to `updates.check_interval`). config-rs lowercases env var keys after prefix stripping (or use `convert_case` feature for explicit control). Unknown `ASTROUP_` env vars that don't map to a config field are silently ignored (unlike unknown TOML keys which produce a warning — this asymmetry is intentional since env vars may be set by unrelated processes).
- **FR-003**: System MUST resolve the default config file path as `{config_dir}/astro-up/config.toml` using platform-aware directory resolution
- **FR-004**: System MUST support these path tokens in config values: `{config_dir}`, `{cache_dir}`, `{data_dir}`, `{home_dir}`. Note: `{program_dir}` is per-package (resolved during detection, not config) and is NOT a config-level token.
- **FR-005**: System MUST validate all configuration values after loading and merging, reporting errors in the format `"config.{section}.{field}: {constraint}, got {actual_value}"` (e.g., `"config.network.timeout: expected positive duration, got -5s"`). Type mismatches (e.g., `timeout = true` instead of a duration string) are reported the same way.
- **FR-006**: System MUST provide these configuration sections: catalog (source URL, cache TTL, offline flag), paths (download, cache, data directories), network (proxy, timeout, user agent), updates (check interval, auto-check enabled), logging (level, log-to-file flag, log file path), telemetry (opt-in flag). Note: downstream specs will add fields to these sections (e.g., `network.download_speed_limit` from spec 010, `backup.retention_count` from spec 013, `ui.*` from spec 016) — the config system MUST support adding new sections and fields without breaking existing configs.
- **FR-007**: System MUST allow the config file path to be overridden via `--config` CLI argument
- **FR-008**: `config show` MUST serialize the current effective configuration (all layers merged, tokens expanded to absolute paths) to TOML. `config init` MUST generate a fully-documented TOML file with all settings commented out at their default values, including explanatory comments per section. `config init` MUST fail with an error if the config file already exists (use `--force` to overwrite).
- **FR-009**: System MUST operate with the defaults documented in the Default Values table when no config file exists and no environment variables are set
- **FR-010**: System MUST log a warning for unknown TOML keys to catch typos, but continue loading (do not fail). Implementation note: config-rs does not natively detect unknown keys — requires a two-pass approach: parse TOML to raw `toml::Value` table, diff keys against the known `AppConfig` fields, warn on unrecognized keys, then proceed with normal config-rs deserialization.
- **FR-011**: System MUST support boolean, integer, string, duration, and path types in config values. Duration values MUST use human-readable strings (e.g., `"24h"`, `"30s"`, `"500ms"`) via `humantime-serde`.
- **FR-012**: System MUST expand path tokens (`{config_dir}` etc.) at config load time, not at usage time. Implementation note: config-rs has no built-in token expansion — this requires a post-processing step after `Config::builder().build()` and before garde validation.

### Key Entities

- **AppConfig**: Top-level configuration struct containing all sections. Serializable to/from TOML. Validatable.
- **CatalogConfig**: `url` (source URL), `cache_ttl` (hard expiry — re-fetch after TTL, no stale-while-revalidate), `offline` (skip catalog network requests only — does not affect downloads or version checks in other specs)
- **PathsConfig**: `download_dir`, `cache_dir`, `data_dir`
- **NetworkConfig**: `proxy` (URL with optional embedded credentials), `timeout`, `user_agent`
- **UpdateConfig**: `auto_check` (enabled flag), `check_interval` (how often to check for astro-up self-updates — distinct from catalog `cache_ttl` which controls catalog freshness)
- **LogConfig**: `level` (error/warn/info/debug/trace), `log_to_file`, `log_file`
- **TelemetryConfig**: `enabled` (opt-in flag)

Note: The catalog signature verification public key is hardcoded, not configurable — allowing users to change it would defeat signature verification.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Application starts and operates correctly with zero configuration (no file, no env vars)
- **SC-002**: Any config value can be overridden via environment variable without modifying files
- **SC-003**: Configuration validation catches all invalid values and reports actionable error messages within 100ms of startup
- **SC-004**: `config init` generates a documented TOML file with all available settings and their defaults
- **SC-005**: Round-trip test passes: load defaults → serialize to TOML → load from TOML → assert equality. Note: `config show` outputs expanded absolute paths, so round-trip equality is tested on the pre-expansion config (tokens preserved in serialization, expanded only in the effective config).
- **SC-006**: Layering precedence test passes: set the same field at all four layers with different values, assert CLI value wins. Remove CLI layer, assert env var wins. Remove env layer, assert TOML wins. Remove TOML, assert default wins.

## Default Values

| Section | Key | Default |
|---------|-----|---------|
| catalog | url | `https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db` |
| catalog | cache_ttl | 24 hours |
| catalog | offline | false |
| paths | download_dir | `{cache_dir}/astro-up/downloads` |
| paths | cache_dir | `{cache_dir}/astro-up` |
| paths | data_dir | `{data_dir}/astro-up` |
| network | proxy | none |
| network | timeout | 30 seconds |
| network | user_agent | `astro-up/{version}` |
| updates | auto_check | true |
| updates | check_interval | 24 hours |
| logging | level | info |
| logging | log_to_file | false |
| logging | log_file | `{data_dir}/astro-up/astro-up.log` |
| telemetry | enabled | false |

## Assumptions

- The application runs on Windows as the primary platform, with macOS/Linux used for development and CI
- The `directories` crate (or equivalent) provides reliable platform-aware path resolution on all supported platforms
- No secrets stored in config for v1 — no GitHub token, no keychain integration. Proxy auth credentials are embedded in the proxy URL. GitHub API token deferred to spec 014 (custom tools).
- Config file is human-edited TOML — no GUI config editor in this spec (that's part of spec 016/017)
- Telemetry is opt-in by default (disabled unless explicitly enabled)
- Concurrent config reads are safe (config is loaded once at startup, immutable after). No file locking needed.

## Dependencies

- **Spec 020** (manifest modernization): The default catalog URL (`catalog.db` on GitHub Releases) depends on the manifest repo publishing this artifact. If spec 020 changes the artifact name or location, the default here must update.
- **Specs 010, 013, 016**: Will add config fields to sections defined here. The config system must be extensible without breaking existing configs (serde `#[serde(default)]`).
