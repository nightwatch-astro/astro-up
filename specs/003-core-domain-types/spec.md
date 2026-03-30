# Feature Specification: Core Domain Types

**Feature Branch**: `003-core-domain-types`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 002 — shared types, traits, and error types for `astro-up-core`

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Software Metadata Model (Priority: P1)

A developer working on any downstream spec (catalog, detection, install, CLI, GUI) imports the `Software` struct from `astro-up-core` and uses it to represent a software package. The struct includes all metadata fields (name, category, publisher, license, tags) plus nested configs (detection, install, checkver, dependencies, hardware, backup). It deserializes from TOML (manifest format) and JSON (SQLite JSON columns) interchangeably via serde.

**Why this priority**: Every other spec depends on these types. The Software struct is the central data model.

**Independent Test**: Deserialize a NINA manifest TOML into a `Software` struct. Serialize it to JSON. Deserialize the JSON back. Assert round-trip equality.

**Acceptance Scenarios**:

1. **Given** a NINA manifest TOML file, **When** deserialized into a `Software` struct, **Then** all fields are populated correctly including nested detection, install, and checkver configs
2. **Given** a `Software` struct, **When** serialized to JSON and back, **Then** the result is identical (round-trip)
3. **Given** a manifest with optional fields omitted (no hardware, no backup), **When** deserialized, **Then** optional fields are `None` and the struct is valid

---

### User Story 2 - Enums and Type Safety (Priority: P2)

A developer uses strongly-typed enums for Category, SoftwareType, DetectionMethod, InstallMethod, Scope, Elevation, and UpgradeBehavior. Each enum derives `Display`, `EnumString`, and serde traits. Invalid values fail at deserialization time with clear error messages, not at runtime deep in business logic.

**Why this priority**: Type safety at the boundary prevents entire classes of bugs. Enums are used everywhere — detection, install, UI display.

**Independent Test**: Deserialize `"category": "capture"` into `Category::Capture`. Try `"category": "invalid"` — expect a serde error.

**Acceptance Scenarios**:

1. **Given** a valid category string "capture", **When** deserialized, **Then** it produces `Category::Capture`
2. **Given** an invalid category string "astronomy", **When** deserialized, **Then** serde returns a descriptive error naming the valid variants
3. **Given** a `Category::Capture` value, **When** displayed, **Then** it renders as `"capture"` (lowercase, matching TOML/JSON format)

---

### User Story 3 - Error Types (Priority: P3)

A developer handling installer failures gets a typed `CoreError::InstallerFailed { exit_code, response }` error that wraps the exit code's semantic meaning (e.g., `KnownExitCode::PackageInUse`). Error types use `thiserror` for the library crate, enabling downstream crates to match on specific error variants without string parsing.

**Why this priority**: Typed errors enable the CLI and GUI to show user-actionable messages ("Close NINA before updating") instead of generic "install failed" messages.

**Independent Test**: Construct an `InstallerFailed` error with exit code 1 for InnoSetup. Assert `response` resolves to `KnownExitCode::PackageInUse`.

**Acceptance Scenarios**:

1. **Given** an installer fails with exit code 1 (InnoSetup), **When** the error is constructed, **Then** `CoreError::InstallerFailed { exit_code: 1, response: KnownExitCode::PackageInUse }` is produced
2. **Given** a `CoreError::NotInstalled` error, **When** formatted with `Display`, **Then** it shows "software not installed"
3. **Given** a `CoreError::ChecksumMismatch { expected, actual }`, **When** formatted, **Then** it shows both hashes for debugging

---

### User Story 4 - Traits for Pluggable Implementations (Priority: P4)

A developer implementing a new detection method (e.g., ASCOM Profile) implements the `Detector` trait. The engine accepts any `Box<dyn Detector>` without knowing the concrete type. Traits define the contract between core and implementation modules. Each trait is async-ready (using async fn in traits, Rust 1.75+).

**Why this priority**: Traits decouple the engine from specific implementations, enabling testing with mocks and adding new detection/install methods without modifying the engine.

**Independent Test**: Create a mock `Detector` that always returns version "1.0.0". Pass it to a function that accepts `&dyn Detector`. Assert it works.

**Acceptance Scenarios**:

1. **Given** a struct implementing `Detector`, **When** passed as `&dyn Detector`, **Then** the engine can call `detect()` without knowing the concrete type
2. **Given** a `Provider` trait, **When** listing releases, **Then** it returns a `Vec<Release>` with version, URL, and asset information
3. **Given** an `Installer` trait implementation, **When** called with `InstallOptions`, **Then** it receives the asset path, config, and quiet/interactive flag

---

### User Story 5 - Event System (Priority: P5)

The engine emits typed events (check_started, download_progress, install_complete, error, etc.) through a channel. Both the CLI (ratatui progress bars) and GUI (Tauri events to Vue frontend) consume these events. The event enum uses Rust's tagged union (enum with data) instead of a struct with a type discriminator field.

**Why this priority**: The event system is the bridge between core logic and both UI layers. Without it, neither CLI nor GUI can show progress.

**Independent Test**: Create an event channel. Emit a `DownloadProgress` event with 50% progress. Receive it and assert the fields.

**Acceptance Scenarios**:

1. **Given** a download in progress, **When** the engine emits `Event::DownloadProgress`, **Then** it carries `software_id`, `progress` (0.0-1.0), `bytes_downloaded`, `total_bytes`, `speed`
2. **Given** an install completes, **When** the engine emits `Event::InstallComplete`, **Then** it carries `software_id` and `message`
3. **Given** an error occurs, **When** the engine emits `Event::Error`, **Then** it carries `software_id`, `message`, and the error type is inspectable

---

### User Story 6 - Fallback Ledger Types (Priority: P6)

For software that cannot be detected automatically (firmware files, manual downloads), the ledger stores a manual version record. The `LedgerEntry` struct records package ID, version, source (astro-up, manual, acknowledged), timestamp, and optional notes. The engine checks: auto-detection > ledger > unknown.

**Why this priority**: Addresses deferred issue #347 (local download ledger). Without this, firmware packages and manual downloads have no version tracking.

**Independent Test**: Create a `LedgerEntry` for a firmware package. Serialize to JSON. Deserialize back. Assert equality.

**Acceptance Scenarios**:

1. **Given** a firmware package with no auto-detection, **When** the user acknowledges version "1.2.3", **Then** a `LedgerEntry` with source `Acknowledged` is created
2. **Given** both auto-detection and a ledger entry exist for the same package, **When** they disagree on version, **Then** the system warns the user and prefers auto-detection
3. **Given** a `LedgerEntry` with source `Manual`, **When** displayed, **Then** it shows "(manually tracked)" indicator

---

### Edge Cases

- What happens when a TOML manifest has fields not in the schema? Serde MUST ignore unknown fields (`#[serde(deny_unknown_fields)]` NOT used) to allow forward compatibility.
- What happens when a version string is not valid semver (e.g., "3.1" or "2024.1.15.1")? The `Version` type MUST handle non-semver strings gracefully by storing the raw string alongside the parsed semver (if parseable).
- What happens when a Software struct has no detection config? This is valid for `download_only` install method (firmware). The detection field MUST be optional.
- What happens when two manifests have the same ID? This is a manifest repo concern (compiler validation), not a core type concern. Core types assume unique IDs.

## Requirements *(mandatory)*

### Functional Requirements

**Software metadata:**
- **FR-001**: The `Software` struct MUST include fields: `id` (String, primary key), `slug` (Option), `name`, `software_type`, `category`, `os` (Vec), `description`, `homepage`, `publisher`, `icon_url`, `license`, `license_url`, `aliases` (Vec), `tags` (Vec), `notes`, `docs_url`, `channel`, `min_os_version`
- **FR-002**: The `Software` struct MUST include nested optional configs: `detection` (Option<DetectionConfig>), `install` (Option<InstallConfig>), `checkver` (Option<CheckverConfig>), `dependencies` (Option<DependencyConfig>), `hardware` (Option<HardwareConfig>), `backup` (Option<BackupConfig>), `versioning` (Option<VersioningConfig>)
- **FR-003**: All types MUST derive `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`. Types used in comparisons MUST also derive `PartialEq`.

**Enums:**
- **FR-004**: `SoftwareType` enum: `Application`, `Driver`, `Runtime`, `Database`, `UsbDriver`, `Resource`. Serialized as lowercase snake_case strings.
- **FR-005**: `Category` enum: `Capture`, `Guiding`, `Platesolving`, `Equipment`, `Focusing`, `Planetarium`, `Viewers`, `Prerequisites`, `Usb`, `Driver`. Serialized as lowercase strings. Uses `strum` for `Display`, `EnumString`, `EnumIter`.
- **FR-006**: `DetectionMethod` enum: `Registry`, `PeFile`, `Wmi`, `DriverStore`, `AscomProfile`, `FileExists`, `ConfigFile`. Serialized as snake_case.
- **FR-007**: `InstallMethod` enum: `Exe`, `Msi`, `InnoSetup`, `Nullsoft`, `Wix`, `Burn`, `Zip`, `ZipWrap`, `Portable`, `DownloadOnly`. Serialized as snake_case.
- **FR-008**: `Scope` enum: `Machine`, `User`, `Either`. `Elevation` enum: `Required`, `Prohibited`, `Self_`. `UpgradeBehavior` enum: `Install`, `UninstallPrevious`, `Deny`.
- **FR-008b**: `CheckMethod` enum: `Github`, `Gitlab`, `DirectUrl`, `HttpHead`, `HtmlScrape` (formerly `go_scrape`), `BrowserScrape` (formerly `rod_scrape`), `PeDownload`, `Manual`, `RuntimeScrape`. Serialized as snake_case. Used by `CheckverConfig.provider` and the manifest repo checker for dispatch.

**Detection:**
- **FR-009**: `DetectionConfig` MUST include: `method`, method-specific fields (registry_key, registry_value, file_path, version_regex, product_code, upgrade_code), and optional `fallback` (boxed self-reference for chain).

**Install (winget switch model):**
- **FR-010**: `InstallConfig` MUST include: `method`, `scope`, `elevation`, `upgrade_behavior`, `install_modes` (Vec), `success_codes` (Vec<i32>), `pre_install` (Vec<String>), `post_install` (Vec<String>), and nested `switches` (InstallerSwitches) and `known_exit_codes` (HashMap<String, KnownExitCode> — String keys for TOML compatibility).
- **FR-011**: `InstallerSwitches` MUST include: `silent`, `interactive`, `upgrade` (each Vec<String>), `install_location` (with `<INSTALLPATH>` token), `log` (with `<LOGPATH>` token), `custom` (Vec<String>).
- **FR-012**: `KnownExitCode` enum: `PackageInUse`, `PackageInUseByApplication`, `RebootRequired`, `CancelledByUser`, `AlreadyInstalled`, `MissingDependency`, `DiskFull`, `InsufficientMemory`, `NetworkError`, `ContactSupport`, `RestartRequired`, `SuccessRebootInitiated`. Serialized as snake_case.

**Checkver / Remote:**
- **FR-013**: `CheckverConfig` MUST include: `provider` (Option — github shorthand), `owner`, `repo`, `url`, `regex`, `jsonpath`, `asset_pattern`, `tag_prefix`, `changelog_url`, and nested `autoupdate` (AutoupdateConfig) and `hash` (HashConfig).
- **FR-014**: `AutoupdateConfig` MUST include: `url` (template with `$version` variables), `hash` (HashConfig).
- **FR-015**: `HashConfig` MUST include: `url` (template), `regex`, `jsonpath`, `mode` (extract, json, download).

**Dependencies:**
- **FR-016**: `DependencyConfig` MUST include: `requires` (Vec of `{id, min_version}`), `optional` (Vec<String>).

**Hardware:**
- **FR-017**: `HardwareConfig` MUST include: `vid_pid` (Vec<String> — patterns like "03C3:*"), `device_class`, `inf_provider`.

**Backup / Policy:**
- **FR-018**: `BackupConfig` MUST include: `config_paths` (Vec<String> with `{config_dir}`, `{program_dir}` path tokens).
- **FR-019**: `VersioningConfig` MUST include: `side_by_side` (bool), `major_version_pattern` (Option), `overrides` (HashMap).
- **FR-020**: `UpdatePolicy` MUST include: `default` (PolicyLevel), `per_package` (HashMap<String, PolicyLevel>). `PolicyLevel` enum: `Minor`, `Major`, `Manual`, `None`.

**Error types:**
- **FR-021**: `CoreError` enum using `thiserror`: `NotInstalled`, `ChecksumMismatch { expected, actual }`, `ProviderUnavailable { provider, cause }`, `ManifestInvalid { id, reason }`, `InstallerFailed { exit_code, response: KnownExitCode }`, `ElevationRequired`, `RebootRequired`, `InstallerTimeout { timeout_secs }`, `InstallerBusy`, `PackageInUse { process_name }`, `AlreadyInstalled { id, version }`, `MissingDependency { dep_id }`, `VersionParseFailed { raw, cause }`, `UnsupportedPlatform`, `NotFound { input }`, `ManualDownloadRequired { id, url, cause }`.

**Traits:**
- **FR-022**: `Detector` trait via `#[trait_variant::make(DetectorDyn: Send)]`: `async fn detect(&self, cfg: &DetectionConfig) -> Result<Version, CoreError>` + `fn supports(&self, method: &DetectionMethod) -> bool`. Engine uses `Box<dyn DetectorDyn>`.
- **FR-023**: `Provider` trait via `#[trait_variant::make(ProviderDyn: Send)]`: `fn name(&self) -> &str` + `async fn latest_release(&self, cfg: &CheckverConfig) -> Result<Release, CoreError>` + `async fn list_releases(&self, cfg: &CheckverConfig, limit: usize) -> Result<Vec<Release>, CoreError>`. Engine uses `Box<dyn ProviderDyn>`.
- **FR-024**: `Installer` trait via `#[trait_variant::make(InstallerDyn: Send)]`: `async fn install(&self, opts: &InstallOptions) -> Result<(), CoreError>` + `fn supports(&self, method: &InstallMethod) -> bool`. Engine uses `Box<dyn InstallerDyn>`.
- **FR-025**: `Downloader` trait via `#[trait_variant::make(DownloaderDyn: Send)]`: `async fn download(&self, request: &DownloadRequest, cancel_token: CancellationToken) -> Result<DownloadResult, CoreError>`. Engine uses `Box<dyn DownloaderDyn>`. (Updated by spec 010 — replaced `DownloadOptions` with `DownloadRequest` + `CancellationToken`, returns `DownloadResult`.)
- **FR-026**: `BackupManager` trait via `#[trait_variant::make(BackupManagerDyn: Send)]`: `async fn backup(...)`, `async fn restore(...)`, `async fn list(...)`, `async fn prune(...)`.

**Events:**
- **FR-027**: `Event` enum with `#[serde(tag = "type", content = "data")]` adjacently tagged serialization for clean TypeScript consumption (`{"type": "download_progress", "data": {...}}`): `CheckStarted { id }`, `CheckProgress { id, progress }`, `CheckComplete { id }`, `DownloadStarted { id, url }`, `DownloadProgress { id, progress, bytes_downloaded, total_bytes, speed, elapsed, estimated_remaining }`, `DownloadComplete { id }`, `BackupStarted { id }`, `BackupComplete { id }`, `InstallStarted { id }`, `InstallComplete { id }`, `ManualDownloadRequired { id, url }`, `Error { id, error: String }`, `ScanStarted`, `ScanProgress { progress, current_id }`, `ScanComplete { total_found }`. (Updated by spec 010 — added `elapsed` and `estimated_remaining` to `DownloadProgress`.)
- **FR-028**: Events MUST be `Send + 'static` for use with `tokio::sync::broadcast` channels (multi-producer, multi-consumer with lagged-receiver semantics). (Updated by spec 010 — replaced `flume` with `tokio::sync::broadcast` since all consumers run tokio.)

**Ledger:**
- **FR-029**: `LedgerEntry` struct: `package_id`, `version`, `source` (LedgerSource enum: AstroUp, Manual, Acknowledged), `recorded_at` (chrono DateTime), `notes` (Option).
- **FR-030**: `LedgerEntry` MUST derive serde traits for SQLite storage via `rusqlite`.

**Metrics:**
- **FR-033**: Core types MUST export metric name constants (e.g., `DOWNLOAD_BYTES_TOTAL`, `SCAN_DURATION_SECONDS`, `CHECK_REQUESTS_TOTAL`, `INSTALL_DURATION_SECONDS`, `CACHE_HIT_TOTAL`, `CACHE_MISS_TOTAL`) as `&str` constants in a `metrics` module. Core types do NOT call `metrics::counter!()` — the engine and UI layers register recorders and emit values using these constants.

**Supporting types:**
- **FR-031**: `Release` struct: `version` (Version), `url`, `asset_name`, `sha256` (Option), `release_date` (Option), `changelog` (Option), `pre_release` (bool).
- **FR-032**: `Version` type MUST use lenient semver parsing: coerce common non-semver patterns to semver (4-part `3.1.2.3001` → strip build, suffix `6.6 SP2` → pre-release, 2-part `3.1` → append `.0`). Always preserve the `raw` string for display. Comparison uses coerced semver when available, lexicographic raw comparison as fallback. MUST implement `Ord`, `Display`, `Serialize`, `Deserialize`.

### Key Entities

- **Software**: The central aggregate — a complete software package definition with all nested configs
- **Version**: A parsed version with semver semantics and raw string fallback
- **Release**: A discovered remote version with download URL and metadata
- **Event**: A typed notification from the engine to UI layers
- **LedgerEntry**: A manual version record for undetectable packages
- **CoreError**: The typed error hierarchy for the entire core crate

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 95 existing manifests deserialize into `Software` structs without errors
- **SC-002**: JSON round-trip serialization produces identical output for all types
- **SC-003**: Every error variant produces a user-readable Display message without internal jargon
- **SC-004**: The core crate compiles on macOS, Linux, and Windows with zero `cfg(windows)` in the types module (platform-specific code is in detection/install modules, not types)
- **SC-005**: All enum variants are exhaustively covered in match statements (no wildcard `_` arms in the core crate)

## Clarifications

### Session 2026-03-29

- Q: Async trait dispatch strategy for dyn Detector/Provider/Installer? → A: `trait_variant::make` (dtolnay) — generates native async trait + dyn-safe variant (e.g., `Detector` + `DetectorDyn`). No `async-trait` crate.
- Q: Where do metric name constants live? → A: Core types export `&str` constants in a `metrics` module. Engine/UI call `metrics::counter!()` using these constants. Types don't emit metrics.
- Q: SQLite schema for catalog.db? → A: Pragmatic normalized (Option D, 8 tables). Updated Spec 002 to match. Top-level config fields are columns; only arrays/maps as JSON.
- Q: Version parsing for non-semver strings? → A: Lenient parsing — coerce to semver where possible (4-part strip, suffix as pre-release, 2-part append .0). Keep raw string for display. Semver comparison when available, lexicographic fallback.
- Q: Event enum serialization for Tauri/TypeScript? → A: Adjacently tagged `#[serde(tag = "type", content = "data")]` — produces `{"type": "download_progress", "data": {...}}` for clean frontend `switch`.

## Assumptions

- The `semver` crate handles version parsing; non-semver strings are stored as-is with a `None` parsed field
- Path tokens (`{config_dir}`, `{program_dir}`, `{cache_dir}`) are NOT expanded in the types module — expansion happens in the config module at runtime
- The `Software` struct is the deserialization target for TOML (manifest files). For SQLite, the pragmatic normalized schema (Option D) stores top-level configs in separate tables with proper columns, and only arrays/maps (switches, exit_codes, tags, aliases, vid_pid, config_paths, autoupdate, hash) as JSON in TEXT columns. The `type` field uses `#[serde(rename = "type")]` (Rust keyword conflict).
- Traits use native async fn (Rust 1.75+) with `trait_variant::make` to generate `dyn`-safe variants (e.g., `Detector` for static dispatch, `DetectorDyn` for `Box<dyn DetectorDyn>` in the engine). No `async-trait` crate.
- The event channel type (`tokio::sync::broadcast`) is NOT defined in core types — only the `Event` enum is. Channel construction is the engine's responsibility. `broadcast` chosen over `flume` (original plan) because all consumers run tokio — no need for sync/async dual API. Capacity 64, lagged receivers skip stale progress events. (Updated by spec 010.)
- This spec covers types only. Implementation of trait methods is in separate specs (detection: 005/006, providers: 007, install: 010, download: 009, backup: 012).
