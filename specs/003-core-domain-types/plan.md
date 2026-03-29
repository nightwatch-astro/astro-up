# Implementation Plan: Core Domain Types

**Branch**: `003-core-domain-types` | **Date**: 2026-03-29 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/003-core-domain-types/spec.md`

## Summary

Define all shared types, enums, traits, error types, and event system in `astro-up-core`. This is the foundation crate — every other crate depends on these types. Implements the winget-enriched data model with Rust idioms: `serde` for serialization, `strum` for enum derives, `thiserror` for typed errors, `trait_variant` for async dyn dispatch.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: serde 1, serde_json 1, thiserror 2, strum 0.26, trait-variant 0.1, semver 1, chrono 0.4
**Storage**: N/A (types only — storage is in catalog/engine specs)
**Testing**: cargo test, insta (snapshot), rstest (parameterized), pretty_assertions
**Target Platform**: Cross-platform (zero `cfg(windows)` in this spec)
**Project Type**: Library crate (`astro-up-core`)

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | Types organized as modules: `types/`, `error.rs`, `traits.rs`, `events.rs`, `metrics.rs` |
| II. Platform Awareness | PASS | Zero `cfg(windows)` — all types are platform-independent |
| III. Test-First | PASS | Snapshot tests for serde round-trips, parameterized tests for version parsing |
| IV. Thin Tauri Boundary | PASS | Types live in core, not gui |
| V. Spec-Driven | PASS | Follows speckit workflow |
| VI. Simplicity | PASS | No abstractions beyond what's needed |

## Project Structure

```text
crates/astro-up-core/src/
├── lib.rs                    # Re-exports all public types
├── types/
│   ├── mod.rs                # Software struct + re-exports
│   ├── software.rs           # Software, SoftwareType, Category
│   ├── detection.rs          # DetectionConfig, DetectionMethod
│   ├── install.rs            # InstallConfig, InstallMethod, Scope, Elevation, etc.
│   ├── checkver.rs           # CheckverConfig, CheckMethod, AutoupdateConfig, HashConfig
│   ├── dependency.rs         # DependencyConfig, Dependency
│   ├── hardware.rs           # HardwareConfig
│   ├── backup.rs             # BackupConfig
│   ├── versioning.rs         # VersioningConfig, UpdatePolicy, PolicyLevel
│   └── version.rs            # Version type (lenient semver wrapper)
├── error.rs                  # CoreError enum (thiserror)
├── traits.rs                 # Detector, Provider, Installer, Downloader, BackupManager
├── events.rs                 # Event enum (adjacently tagged for Tauri IPC)
├── ledger.rs                 # LedgerEntry, LedgerSource
├── release.rs                # Release struct
└── metrics.rs                # Metric name constants
```

## Key Technical Decisions

### Serde strategy

All types derive `Serialize` + `Deserialize`. Enums use `#[serde(rename_all = "snake_case")]` for consistent serialization. The `Software` struct uses `#[serde(default)]` on optional fields so TOML manifests with missing sections still deserialize. Unknown fields are silently ignored (no `deny_unknown_fields`).

### Strum for enums

All enums derive `strum::Display`, `strum::EnumString`, `strum::EnumIter` with `#[strum(serialize_all = "snake_case")]`. This gives us `FromStr`, `Display`, and iteration for free — used by CLI (parsing user input), GUI (dropdowns), and catalog (filtering).

### Lenient Version parsing

```rust
pub struct Version {
    pub raw: String,
    pub parsed: Option<semver::Version>,
}
```

Parsing: try `semver::Version::parse()` first. If that fails, try coercion (strip 4th component, convert suffix to pre-release). Always store `raw`. `Ord` compares `parsed` when both are `Some`, falls back to lexicographic `raw` comparison.

### trait_variant for async traits

```rust
#[trait_variant::make(DetectorDyn: Send)]
pub trait Detector {
    async fn detect(&self, cfg: &DetectionConfig) -> Result<Version, CoreError>;
    fn supports(&self, method: &DetectionMethod) -> bool;
}
```

Engine uses `Vec<Box<dyn DetectorDyn>>`. Implementations write `impl Detector for X`.

### Event adjacently tagged

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event { ... }
```

Frontend receives `{"type": "download_progress", "data": {"id": "nina-app", ...}}`.

## Dependencies (additions to workspace)

| Crate | Version | Purpose |
|-------|---------|---------|
| strum | 0.26 | Enum derive macros (Display, EnumString, EnumIter) |
| strum_macros | 0.26 | Strum proc macros |
| trait-variant | 0.1 | Async trait dyn dispatch |
| semver | 1 (serde) | Version parsing |
| chrono | 0.4 (serde) | Timestamps for LedgerEntry |
| rstest | 0.23 | Parameterized tests (dev-dep) |
