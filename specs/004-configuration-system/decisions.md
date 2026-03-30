# Decisions Report: 004-configuration-system

**Created**: 2026-03-29
**Updated**: 2026-03-30 (pivot to SQLite)

## Decisions Made

### D1: SQLite over TOML file for config storage
**Choice**: SQLite key-value table (rusqlite, already in workspace)
**Reasoning**: The GUI settings page (spec 016/017) needs to write config. TOML files lose comments on programmatic write, require token expansion for path fields, and env var layering adds complexity for a desktop app that doesn't run in containers. SQLite is already in the stack (catalog, ledger), both CLI and GUI share the same database, and key-value storage is simpler than file parsing. Aligned with constitution tech stack which lists "config" under SQLite.
**Alternatives**: TOML file + config-rs (original design — 4 layers, token expansion, unknown key detection, env var mapping), figment (similar complexity), JSON file (no standard for Rust config)
**Previous**: D1 originally chose figment, then config-rs. Both dropped in favor of SQLite.

### D3: garde over validator for validation
**Choice**: garde v0.22 (derive + url features)
**Reasoning**: Cleaner derive syntax, automatic nested validation with `#[garde(dive)]`. Consistent with migration plan.
**Alternatives**: validator crate (more verbose)

### D6: Telemetry opt-in by default
**Choice**: Disabled unless user explicitly enables
**Reasoning**: Privacy-first for astrophotography. Users controlling expensive equipment shouldn't have unexpected network traffic. GDPR-safe default.

### D7: No GUI config editor in this spec
**Choice**: Defer GUI settings page to spec 016/017
**Reasoning**: This spec covers the config system (storage, validation, API). The settings UI depends on this API but belongs in the frontend spec.

### D8: Single SQLite database file
**Choice**: Config shares the same .db as catalog and ledger
**Reasoning**: One file to manage, back up, and migrate. All data is app-local. Config is just another table (`config_settings`) alongside catalog and ledger tables.
**Alternatives**: Separate config.db (unnecessary complexity, two files to manage)

### D9: Three-layer precedence (drop env vars)
**Choice**: Compiled defaults → SQLite → CLI flags
**Reasoning**: Env vars add complexity (ASTROUP_ prefix, double underscore nesting, try_parsing) with no practical value for a Windows desktop app. Power users use `config set` for persistent changes or CLI flags for per-invocation overrides. CI uses CLI flags. No Docker/container use case exists.
**Previous**: Original design had 4 layers (defaults → TOML → env vars → CLI). Env vars and TOML both dropped.

### D10: Config module receives db_path, doesn't resolve platform dirs
**Choice**: Platform directory resolution happens at app init, not in config module
**Reasoning**: Keeps config module testable (pass tempfile path), platform-agnostic, and simple. The `directories` crate is used in CLI main.rs / Tauri lib.rs, not in core config.

### D11: humantime for duration parsing (not humantime-serde)
**Choice**: Direct `humantime::parse_duration` / `humantime::format_duration` calls
**Reasoning**: Duration fields are stored as strings in SQLite, parsed explicitly at the API boundary. No serde integration needed. humantime is already a transitive dependency — no new crate.

## Clarify-Phase Decisions (retained)

### C3: GitHub token removed from config — deferred to custom tools
**Decision**: No secrets in config for v1. GitHub API token deferred to spec 014 (custom tools).

### C4: Default values explicitly documented
**Decision**: Defaults are part of the spec (Default Values table), not left to implementation judgment.

### C5: Catalog URL points to GitHub Releases
**Decision**: `https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db`

## Questions (retained)

### Q1: Should config support hot-reloading?
**Decision**: No. Config is loaded once at startup. `config set` persists for next run.

### Q2: Should proxy credentials support keychain?
**Decision**: No, not in this spec. Proxy auth embedded in URL.
