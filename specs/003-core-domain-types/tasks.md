# Tasks: Core Domain Types

**Input**: Design documents from `specs/003-core-domain-types/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)

## Phase 1: Setup

**Purpose**: Add new dependencies to workspace Cargo.toml, create module structure

- [ ] T001 Add workspace dependencies to `Cargo.toml`: `strum = { version = "0.26", features = ["derive"] }`, `trait-variant = "0.1"`, `semver = { version = "1", features = ["serde"] }`, `chrono = { version = "0.4", features = ["serde"] }`, `rstest = "0.23"` (dev)
- [ ] T002 Add dependencies to `crates/astro-up-core/Cargo.toml`: `strum.workspace = true`, `trait-variant.workspace = true`, `semver.workspace = true`, `chrono.workspace = true`, and `rstest.workspace = true` (dev)
- [ ] T003 Create module structure: `crates/astro-up-core/src/types/mod.rs`, `types/software.rs`, `types/detection.rs`, `types/install.rs`, `types/checkver.rs`, `types/dependency.rs`, `types/hardware.rs`, `types/backup.rs`, `types/versioning.rs`, `types/version.rs` as empty files with module declarations

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Version type and enums needed by all user stories

- [ ] T004 Implement `Version` type in `crates/astro-up-core/src/types/version.rs`: struct with `raw: String` + `parsed: Option<semver::Version>`, `Version::parse()` with lenient coercion (strip 4th component, pad missing, suffix as pre-release), impl `Ord` (parsed comparison with raw fallback), impl `Display` (show raw), derive `Serialize`/`Deserialize` (serialize as raw string), impl `From<&str>` and `FromStr`
- [ ] T005 [P] Implement `SoftwareType` enum in `crates/astro-up-core/src/types/software.rs`: `Application`, `Driver`, `Runtime`, `Database`, `UsbDriver`, `Resource` with `#[serde(rename_all = "snake_case")]` + `#[strum(serialize_all = "snake_case")]` + derive `Display`, `EnumString`, `EnumIter`, `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`
- [ ] T006 [P] Implement `Category` enum in `crates/astro-up-core/src/types/software.rs`: `Capture`, `Guiding`, `Platesolving`, `Equipment`, `Focusing`, `Planetarium`, `Viewers`, `Prerequisites`, `Usb`, `Driver` — same derives as SoftwareType
- [ ] T007 [P] Implement `DetectionMethod` enum in `crates/astro-up-core/src/types/detection.rs`: `Registry`, `PeFile`, `Wmi`, `DriverStore`, `AscomProfile`, `FileExists`, `ConfigFile` — same derives
- [ ] T008 [P] Implement `InstallMethod` enum in `crates/astro-up-core/src/types/install.rs`: `Exe`, `Msi`, `InnoSetup`, `Nullsoft`, `Wix`, `Burn`, `Zip`, `ZipWrap`, `Portable`, `DownloadOnly` — same derives
- [ ] T009 [P] Implement `Scope`, `Elevation`, `UpgradeBehavior` enums in `crates/astro-up-core/src/types/install.rs`
- [ ] T010 [P] Implement `CheckMethod` enum in `crates/astro-up-core/src/types/checkver.rs`: `Github`, `Gitlab`, `DirectUrl`, `HttpHead`, `HtmlScrape`, `BrowserScrape`, `PeDownload`, `Manual`, `RuntimeScrape` — same derives
- [ ] T011 [P] Implement `KnownExitCode` enum in `crates/astro-up-core/src/types/install.rs`: `PackageInUse`, `PackageInUseByApplication`, `RebootRequired`, `CancelledByUser`, `AlreadyInstalled`, `MissingDependency`, `DiskFull`, `InsufficientMemory`, `NetworkError`, `ContactSupport`, `RestartRequired`, `SuccessRebootInitiated` — same derives
- [ ] T012 Verify all enums compile and round-trip with `cargo test -p astro-up-core`

**Checkpoint**: All enums and Version type compile with serde + strum derives.

---

## Phase 3: User Story 1 — Software Metadata Model (Priority: P1)

**Goal**: `Software` struct deserializes from TOML and JSON, round-trips correctly

**Independent Test**: Deserialize NINA manifest TOML → Software → JSON → Software. Assert equality.

- [ ] T013 [US1] Implement `DetectionConfig` struct in `crates/astro-up-core/src/types/detection.rs`: `method`, `registry_key`, `registry_value`, `file_path`, `version_regex`, `product_code`, `upgrade_code` (all Option except method), `fallback: Option<Box<DetectionConfig>>`
- [ ] T014 [P] [US1] Implement `InstallerSwitches` struct in `crates/astro-up-core/src/types/install.rs`: `silent`, `interactive`, `upgrade` (Vec<String>), `install_location`, `log`, `custom`
- [ ] T015 [P] [US1] Implement `InstallConfig` struct in `crates/astro-up-core/src/types/install.rs`: `method`, `scope`, `elevation`, `upgrade_behavior`, `install_modes`, `success_codes`, `pre_install`, `post_install`, `switches: Option<InstallerSwitches>`, `known_exit_codes: HashMap<i32, KnownExitCode>`
- [ ] T016 [P] [US1] Implement `HashConfig` and `AutoupdateConfig` in `crates/astro-up-core/src/types/checkver.rs`
- [ ] T017 [P] [US1] Implement `CheckverConfig` struct in `crates/astro-up-core/src/types/checkver.rs`: `provider`, `owner`, `repo`, `url`, `regex`, `jsonpath`, `asset_pattern`, `tag_prefix`, `changelog_url`, `autoupdate`, `hash`
- [ ] T018 [P] [US1] Implement `DependencyConfig` and `Dependency` in `crates/astro-up-core/src/types/dependency.rs`: requires (Vec<Dependency> with id + min_version), optional (Vec<String>)
- [ ] T019 [P] [US1] Implement `HardwareConfig` in `crates/astro-up-core/src/types/hardware.rs`: vid_pid, device_class, inf_provider
- [ ] T020 [P] [US1] Implement `BackupConfig` in `crates/astro-up-core/src/types/backup.rs`: config_paths with path tokens
- [ ] T021 [P] [US1] Implement `VersioningConfig` and `UpdatePolicy` in `crates/astro-up-core/src/types/versioning.rs`: side_by_side, major_version_pattern, overrides, PolicyLevel enum
- [ ] T022 [US1] Implement `Software` struct in `crates/astro-up-core/src/types/software.rs`: all metadata fields + nested Option configs. Use `#[serde(default)]` for optional sections.
- [ ] T023 [US1] Create `crates/astro-up-core/src/types/mod.rs` re-exporting all types
- [ ] T024 [US1] Add snapshot test in `crates/astro-up-core/src/types/software.rs`: deserialize a NINA-like TOML manifest into Software, snapshot with insta. Add JSON round-trip assertion.
- [ ] T025 [US1] Add snapshot test for a minimal manifest (only required fields, no optional sections)

**Checkpoint**: Software struct deserializes from TOML, round-trips to/from JSON.

---

## Phase 4: User Story 2 — Enums and Type Safety (Priority: P2)

**Goal**: Invalid enum values fail at deserialization time with descriptive errors

**Independent Test**: Attempt to deserialize invalid category string — expect serde error with valid variants listed.

- [ ] T026 [US2] Add parameterized tests in `crates/astro-up-core/src/types/software.rs` using `rstest`: valid category strings ("capture" → Capture, "driver" → Driver, etc.) and invalid strings ("astronomy" → error)
- [ ] T027 [P] [US2] Add parameterized tests for `DetectionMethod`, `InstallMethod`, `CheckMethod` round-trips
- [ ] T028 [US2] Add test that `Category::iter()` returns all 10 variants (strum EnumIter)

**Checkpoint**: All enum deserialization validated with parameterized tests.

---

## Phase 5: User Story 3 — Error Types (Priority: P3)

**Goal**: Typed CoreError hierarchy with user-readable Display messages

**Independent Test**: Construct each error variant, assert Display output is human-readable.

- [ ] T029 [US3] Implement `CoreError` enum in `crates/astro-up-core/src/error.rs` using `thiserror`: all variants from FR-021 with `#[error("...")]` messages. Include `#[from]` for common conversions (io::Error, serde_json::Error, semver::Error).
- [ ] T030 [US3] Add tests for each CoreError variant: construct, format with Display, assert message is user-readable (no internal jargon). Snapshot test the full set of error messages with insta.

**Checkpoint**: All error variants produce clear user-facing messages.

---

## Phase 6: User Story 4 — Traits (Priority: P4)

**Goal**: Async traits with dyn-safe variants via trait_variant

**Independent Test**: Create a mock Detector, pass as &dyn DetectorDyn, call detect().

- [ ] T031 [US4] Implement traits in `crates/astro-up-core/src/traits.rs`: `Detector` (detect + supports), `Provider` (name + latest_release + list_releases), `Installer` (install + supports), `Downloader` (download), `BackupManager` (backup + restore + list + prune) — each with `#[trait_variant::make(XxxDyn: Send)]`
- [ ] T032 [P] [US4] Implement supporting types: `InstallOptions` struct (asset_path, config, quiet), `DownloadOptions` struct (on_progress callback, checksum, resume)
- [ ] T033 [US4] Add test: create a mock struct implementing Detector, pass as `Box<dyn DetectorDyn>`, call detect(). Verify it compiles and runs.

**Checkpoint**: All traits compile with trait_variant, dyn dispatch verified.

---

## Phase 7: User Story 5 — Event System (Priority: P5)

**Goal**: Adjacently tagged Event enum serializes for Tauri IPC

**Independent Test**: Serialize DownloadProgress event to JSON, verify `{"type": "download_progress", "data": {...}}` format.

- [ ] T034 [US5] Implement `Event` enum in `crates/astro-up-core/src/events.rs` with `#[serde(tag = "type", content = "data")]` and `#[serde(rename_all = "snake_case")]`: all variants from FR-027. Derive `Serialize`, `Deserialize`, `Debug`, `Clone`. Assert `Send + 'static`.
- [ ] T035 [US5] Add snapshot tests for Event serialization: serialize each variant to JSON, verify adjacently tagged format with insta.

**Checkpoint**: Events serialize to `{"type": "...", "data": {...}}` for Tauri IPC.

---

## Phase 8: User Story 6 — Ledger Types (Priority: P6)

**Goal**: LedgerEntry for manual version tracking

**Independent Test**: Create LedgerEntry, serialize to JSON, deserialize back.

- [ ] T036 [US6] Implement `LedgerSource` enum (`AstroUp`, `Manual`, `Acknowledged`) and `LedgerEntry` struct in `crates/astro-up-core/src/ledger.rs`: package_id, version (Version), source, recorded_at (chrono DateTime<Utc>), notes (Option)
- [ ] T037 [P] [US6] Implement `Release` struct in `crates/astro-up-core/src/release.rs`: version (Version), url, asset_name, sha256 (Option), release_date (Option), changelog (Option), pre_release (bool)
- [ ] T038 [US6] Add round-trip tests for LedgerEntry and Release serialization

**Checkpoint**: Ledger and Release types serialize/deserialize correctly.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Metrics constants, lib.rs exports, final verification

- [ ] T039 [P] Implement metric name constants in `crates/astro-up-core/src/metrics.rs`: `DOWNLOAD_BYTES_TOTAL`, `SCAN_DURATION_SECONDS`, `CHECK_REQUESTS_TOTAL`, `INSTALL_DURATION_SECONDS`, `CACHE_HIT_TOTAL`, `CACHE_MISS_TOTAL` as `pub const &str`
- [ ] T040 Update `crates/astro-up-core/src/lib.rs`: add `pub mod types`, `pub mod error`, `pub mod traits`, `pub mod events`, `pub mod ledger`, `pub mod release`, `pub mod metrics`. Remove the old `version()` function (moved to types or keep as convenience re-export).
- [ ] T041 Run `cargo clippy -p astro-up-core -- -D warnings` and fix any warnings
- [ ] T042 Run `cargo test -p astro-up-core` and verify all tests pass
- [ ] T043 Run `just check` to verify workspace-wide quality gates still pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup (T001-T003)
- **US1 (Phase 3)**: Depends on Foundational (enums + Version needed by config structs)
- **US2 (Phase 4)**: Depends on US1 (tests need full Software struct)
- **US3 (Phase 5)**: Depends on Foundational (errors reference enum types)
- **US4 (Phase 6)**: Depends on US1 (traits reference config types)
- **US5 (Phase 7)**: Independent of US1-US4 (Event only uses String IDs)
- **US6 (Phase 8)**: Depends on Foundational (uses Version type)
- **Polish (Phase 9)**: Depends on all user stories

### Parallel Opportunities

Within Phase 2 (after T004 Version):
```
T005 (SoftwareType) | T006 (Category) | T007 (DetectionMethod) | T008 (InstallMethod) | T009 (Scope/Elevation) | T010 (CheckMethod) | T011 (KnownExitCode)
```

Within Phase 3:
```
T013 (DetectionConfig) | T014 (InstallerSwitches) | T015 (InstallConfig) | T016 (HashConfig) | T017 (CheckverConfig) | T018 (DependencyConfig) | T019 (HardwareConfig) | T020 (BackupConfig) | T021 (VersioningConfig)
```

Within Phase 5 + 7 + 8 (independent stories):
```
US3 (errors) | US5 (events) | US6 (ledger+release) — can run in parallel
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Setup (T001-T003)
2. Foundational enums + Version (T004-T012)
3. US1: Software struct + config types (T013-T025)
4. **STOP and VALIDATE**: NINA manifest deserializes correctly

### Incremental Delivery

1. Setup + Foundational → enums compile, Version parses
2. US1 → Software struct round-trips TOML ↔ JSON (MVP)
3. US2 → Enum validation tests
4. US3 → Error types with Display messages
5. US4 → Traits with dyn dispatch
6. US5 → Events for Tauri IPC
7. US6 → Ledger + Release types
8. Polish → metrics, lib.rs exports, final checks
