# Decisions Report: 004-configuration-system

**Created**: 2026-03-29
**Mode**: Unattended — decisions made autonomously

## Decisions Made

### D1: config-rs over figment for layered configuration
**Choice**: config-rs (`config` crate)
**Reasoning**: config-rs is the most popular Rust config library (~3.5M downloads/month, 4x figment). It has practical advantages for our use case: `try_parsing(true)` auto-coerces env var strings to ints/bools, `.list_separator(",")` parses comma-separated env vars into `Vec`, typed `ConfigError` variants (Type, NotFound, FileParse) with origin tracking aid our FR-005 error format requirement, and `Environment::source(Some(hashmap))` enables clean test mocking without filesystem sandboxes. Both libraries have the same gaps (no unknown key detection, no path token expansion, no clap integration), but config-rs has more batteries included for the common cases.
**Alternatives considered**: figment (cleaner API, first-class profiles, `Jail` test helper — but profiles not needed, and fewer practical features for env var handling), manual layering with serde (more control but more code)

### D2: Double underscore for nested env var mapping
**Choice**: `ASTROUP_CATALOG__URL` maps to `catalog.url`
**Reasoning**: config-rs's `Environment::separator("__")` uses `__` for nesting, which is a common convention (also used by Docker, Kubernetes). Single underscore is ambiguous for multi-word field names (e.g., `check_interval` vs nested `check.interval`).
**Alternatives considered**: Single underscore with explicit mapping (more fragile), JSON in env var (poor UX)

### D3: garde over validator for validation
**Choice**: garde (already in the migration plan's dependency list)
**Reasoning**: Migration plan explicitly chose garde over validator for its cleaner derive syntax and automatic nested validation. Consistent with the rest of the project.
**Alternatives considered**: validator crate (migration plan initially mentioned it but switched to garde)

### D4: Warn on unknown TOML keys, don't error
**Choice**: Warn (log warning) but don't fail
**Reasoning**: Strict rejection would break forward compatibility — if a user has a newer config file and downgrades, unknown keys would prevent startup. Warning catches typos while remaining tolerant.
**Update**: Changed FR-010 to "reject with warning" — this means log a warning, not crash. Clarified in spec.

### D5: Path token expansion at load time
**Choice**: Expand `{config_dir}` etc. when config is loaded, not lazily at point of use
**Reasoning**: Fail-fast — if a token can't be resolved, we want to know at startup, not deep in a download operation. Also simplifies downstream code (all paths are fully resolved strings).
**Alternatives considered**: Lazy expansion (would defer errors but complicate path handling everywhere)

### D6: Telemetry opt-in by default
**Choice**: Disabled unless user explicitly enables
**Reasoning**: Privacy-first for an astrophotography tool. Users controlling expensive equipment shouldn't have unexpected network traffic. Opt-in is also the GDPR-safe default.

### D7: No GUI config editor in this spec
**Choice**: Defer to spec 016 (Vue Frontend)
**Reasoning**: This spec covers the configuration system (loading, validation, serialization). The GUI settings page is a frontend concern that depends on this spec's API but belongs in the frontend spec.

## Clarify-Phase Decisions

### C1: `{program_dir}` is NOT a config-level token
**Finding**: `{program_dir}` varies per package (e.g., `C:\Program Files\NINA` vs `C:\Program Files\PHD2`). It's resolved during detection (spec 006), not at config load time.
**Decision**: Removed from FR-004. Config tokens are: `{config_dir}`, `{cache_dir}`, `{data_dir}`, `{home_dir}`.
**Impact**: Manifest `[backup]` and `[detection]` sections still use `{program_dir}` — but that's resolved per-package by the detection system, not by config.

### C2: Config file forward-compatibility via serde defaults
**Finding**: No explicit config version field needed. Serde's `#[serde(default)]` handles missing fields (new fields get defaults on old configs). Unknown fields get a warning (FR-010). This handles both upgrades and downgrades gracefully.
**Decision**: No migration system. No version field. Rely on serde defaults + warn-on-unknown.

### C3: GitHub token removed from config — deferred to custom tools
**Finding**: The GitHub token was for rate-limited API calls during version checking. But official packages get their versions from the catalog (one HTTP request, no API). Only custom tools (spec 014) would need per-repo GitHub API calls, and most users will have <5 custom tools (well within the 60/hour unauthenticated limit).
**Decision**: Remove github_token from NetworkConfig entirely. If needed later, add as an optional field in spec 014 (custom tools). No secrets in config for v1.

### C4: Default values explicitly documented
**Finding**: Spec said "sensible defaults" without listing them. Added a defaults table to the spec covering all sections (catalog URL, TTL, paths, network, updates, logging, telemetry).
**Decision**: Defaults are now part of the spec, not left to implementation judgment.

### C5: Catalog URL points to GitHub Releases
**Finding**: Default catalog URL uses GitHub Releases latest download pattern. This is stable, CDN-cached, and doesn't require any infrastructure beyond the manifest repo.
**Decision**: `https://github.com/nightwatch-astro/astro-up-manifests/releases/latest/download/catalog.db`

## Questions I Would Have Asked

### Q1: Should config support hot-reloading?
**My decision**: No. Config is loaded once at startup. Hot-reload adds complexity (file watchers, partial re-initialization) with minimal benefit for a desktop app that restarts quickly.
**Impact if wrong**: Low — can be added later without breaking changes.

### Q2: Should proxy credentials support 1Password / system keychain?
**My decision**: No, not in this spec. Proxy auth credentials are embedded in the proxy URL (`http://user:pass@proxy:8080`). Keychain integration is a separate feature.
**Impact if wrong**: Medium — users with credential rotation would need to update the config file.

### Q3: Should `config init` generate a minimal or fully-documented config?
**My decision**: Fully documented — all settings with defaults commented out, explanatory comments for each section. This serves as self-documentation.
**Impact if wrong**: Low — purely UX preference.
