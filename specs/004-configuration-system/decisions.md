# Decisions Report: 004-configuration-system

**Created**: 2026-03-29
**Mode**: Unattended — decisions made autonomously

## Decisions Made

### D1: figment over config-rs for layered configuration
**Choice**: figment
**Reasoning**: figment (by Rocket's author) has first-class support for the exact layering pattern we need (defaults → TOML → env vars → CLI), built-in TOML and env var providers, and clean merge semantics. config-rs is older and less actively maintained. figment also supports custom providers, making it easy to add CLI arg integration.
**Alternatives considered**: config-rs (less active, more boilerplate), manual layering with serde (more control but more code)

### D2: Double underscore for nested env var mapping
**Choice**: `ASTROUP_CATALOG__URL` maps to `catalog.url`
**Reasoning**: figment's default env var provider uses `__` for nesting, which is a common convention (also used by Docker, Kubernetes). Single underscore is ambiguous for multi-word field names (e.g., `check_interval` vs nested `check.interval`).
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

### C3: GitHub token in plaintext is acceptable for v1
**Finding**: The only secret in config is the GitHub API token. Storing it in TOML is plaintext on disk. Alternatives (keychain, 1Password CLI) add OS-specific complexity.
**Decision**: Accept plaintext for v1. Users who want secure storage use `ASTROUP_GITHUB_TOKEN` env var. Document this in `config init` output.

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
