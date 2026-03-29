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

A power user creates a `config.toml` file at `{config_dir}/astro-up/config.toml` to customize behavior. They change the download directory, set a proxy, and configure a GitHub token for higher API rate limits. The TOML values override the defaults but can themselves be overridden by environment variables.

**Why this priority**: TOML config is the primary customization mechanism for persistent settings. Power users expect file-based configuration.

**Independent Test**: Create a config.toml with custom values, launch the application, verify the custom values are used.

**Acceptance Scenarios**:

1. **Given** a config.toml with `download_dir = "D:\\Astro\\Downloads"`, **When** the application loads config, **Then** the download directory resolves to `D:\Astro\Downloads`
2. **Given** a config.toml with `proxy = "http://proxy.local:8080"`, **When** the application makes HTTP requests, **Then** traffic routes through the specified proxy
3. **Given** a config.toml with an invalid value (e.g., negative update interval), **When** the application loads config, **Then** it reports a clear validation error with the field name and constraint

---

### User Story 3 - Environment Variable Overrides (Priority: P3)

A CI/CD system or Docker container runs astro-up with configuration injected via environment variables. The `ASTROUP_` prefix maps to config fields: `ASTROUP_GITHUB_TOKEN` sets the GitHub API token, `ASTROUP_CATALOG__URL` sets the catalog source URL (double underscore for nesting). Environment variables take highest precedence after CLI arguments.

**Why this priority**: Environment variables enable headless/automated operation and secret injection without config files on disk.

**Independent Test**: Set `ASTROUP_GITHUB_TOKEN=ghp_test123`, launch the application, verify the token is used for GitHub API requests.

**Acceptance Scenarios**:

1. **Given** `ASTROUP_GITHUB_TOKEN=ghp_xxx` is set, **When** the application loads config, **Then** the GitHub token is `ghp_xxx` regardless of what the TOML file says
2. **Given** `ASTROUP_CATALOG__URL=https://custom.example.com/catalog.db` is set, **When** the application loads config, **Then** the catalog URL points to the custom location
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
- What happens when the GitHub token is stored in plaintext in config.toml? This is accepted for v1. Users who want secure storage should use `ASTROUP_GITHUB_TOKEN` env var instead. Keychain integration is deferred.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST load configuration in this precedence order (highest to lowest): CLI arguments → environment variables → TOML config file → compiled defaults
- **FR-002**: System MUST use the `ASTROUP_` prefix for environment variable mapping, with double underscore for nesting (e.g., `ASTROUP_CATALOG__URL` maps to `catalog.url`)
- **FR-003**: System MUST resolve the default config file path as `{config_dir}/astro-up/config.toml` using platform-aware directory resolution
- **FR-004**: System MUST support these path tokens in config values: `{config_dir}`, `{cache_dir}`, `{data_dir}`, `{home_dir}`. Note: `{program_dir}` is per-package (resolved during detection, not config) and is NOT a config-level token.
- **FR-005**: System MUST validate all configuration values after loading and merging, reporting the field name and constraint on failure
- **FR-006**: System MUST provide these configuration sections: catalog (source URLs, cache TTL), paths (download, cache, data directories), network (proxy, timeouts, GitHub token), updates (check interval, auto-check enabled), logging (level, file path), telemetry (opt-in flag)
- **FR-007**: System MUST allow the config file path to be overridden via `--config` CLI argument
- **FR-008**: System MUST serialize the current effective configuration to TOML for `config show` and `config init` commands
- **FR-009**: System MUST operate with sensible defaults when no config file exists and no environment variables are set
- **FR-010**: System MUST log a warning for unknown TOML keys to catch typos, but continue loading (do not fail)
- **FR-011**: System MUST support boolean, integer, string, duration, and path types in config values
- **FR-012**: System MUST expand path tokens (`{config_dir}` etc.) at config load time, not at usage time

### Key Entities

- **AppConfig**: Top-level configuration struct containing all sections. Serializable to/from TOML. Validatable.
- **CatalogConfig**: Catalog source URL, signature verification public key, cache TTL, offline mode flag
- **PathsConfig**: Download directory, cache directory, data directory, log file path
- **NetworkConfig**: HTTP proxy URL, request timeout, GitHub API token, user agent string
- **UpdateConfig**: Auto-check enabled, check interval, allowed update channels
- **LogConfig**: Log level (error/warn/info/debug/trace), log to file flag, log file path
- **TelemetryConfig**: Opt-in flag, anonymous usage metrics endpoint

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Application starts and operates correctly with zero configuration (no file, no env vars)
- **SC-002**: Any config value can be overridden via environment variable without modifying files
- **SC-003**: Configuration validation catches all invalid values and reports actionable error messages within 100ms of startup
- **SC-004**: `config init` generates a documented TOML file with all available settings and their defaults
- **SC-005**: Round-trip test passes: load defaults → serialize to TOML → load from TOML → assert equality

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
| network | github_token | none |
| updates | auto_check | true |
| updates | check_interval | 24 hours |
| logging | level | info |
| logging | log_to_file | false |
| logging | log_file | `{data_dir}/astro-up/astro-up.log` |
| telemetry | enabled | false |

## Assumptions

- The application runs on Windows as the primary platform, with macOS/Linux used for development and CI
- The `directories` crate (or equivalent) provides reliable platform-aware path resolution on all supported platforms
- GitHub tokens are the only secret stored in config; proxy auth credentials are embedded in the proxy URL
- Config file is human-edited TOML — no GUI config editor in this spec (that's part of the frontend spec)
- Telemetry is opt-in by default (disabled unless explicitly enabled)
