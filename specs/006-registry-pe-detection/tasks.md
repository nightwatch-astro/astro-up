# Tasks: Software and Driver Detection

**Input**: Design documents from `/specs/006-registry-pe-detection/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/detect-trait.rs

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Phase 1: Setup

**Purpose**: Add dependencies and create module structure

- [ ] T001 Add pelite 0.10 dependency and winreg 0.56 + wmi 0.14 as cfg(windows) dependencies in `crates/astro-up-core/Cargo.toml`
- [ ] T002 Create `crates/astro-up-core/src/detect/mod.rs` with submodule declarations and re-export `pub mod detect` from `crates/astro-up-core/src/lib.rs`
- [ ] T003 [P] Add test PE fixture with known VS_FIXEDFILEINFO version (3.2.1.0) in `crates/astro-up-core/tests/fixtures/test.exe`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types, path resolver, and chain runner that ALL user stories depend on

- [ ] T004 Define `DetectionResult`, `DetectionMethod`, `ScanResult`, `PackageDetection`, `ScanError` types in `crates/astro-up-core/src/detect/mod.rs` per contracts/detect-trait.rs
- [ ] T005 [P] Implement `PathResolver` with platform token expansion (`{program_files}`, `{app_data}`, etc.) in `crates/astro-up-core/src/detect/path.rs` — use `directories` crate for cross-platform paths, `env::var` for Windows-specific tokens behind `cfg(windows)`
- [ ] T006 [P] Implement `DetectionError` error type with `CatalogError`, `LedgerError`, `WmiConnectionError` variants in `crates/astro-up-core/src/detect/mod.rs`
- [ ] T007 Implement detection chain runner in `crates/astro-up-core/src/detect/mod.rs` — execute `DetectionConfig` fallback chain, stop at first `Installed` or `InstalledUnknownVersion`, recurse into `config.fallback`
- [ ] T008 [P] Unit tests for path token expansion (Windows + non-Windows) in `crates/astro-up-core/src/detect/path.rs` (inline tests)
- [ ] T009 [P] Unit tests for chain runner logic (stop-at-first-success, exhaust chain, single method) in `crates/astro-up-core/src/detect/mod.rs` (inline tests) — use mock detection methods returning fixed results

**Checkpoint**: Foundation ready — detection types defined, chain runner works, path resolver expands tokens

---

## Phase 3: User Story 1 — Detect Installed Software via Registry (Priority: P1)

**Goal**: Detect installed astrophotography software via Windows uninstall registry keys (HKLM + HKCU, 32-bit + 64-bit views)

**Independent Test**: Run registry detection for a known package, verify it returns the correct version string

- [ ] T010 [US1] Implement `registry::detect()` in `crates/astro-up-core/src/detect/registry.rs` — open HKLM + HKCU uninstall keys with `KEY_WOW64_64KEY` and `KEY_WOW64_32KEY` via `winreg::RegKey::open_subkey_with_flags`, read `DisplayVersion` (or configurable value from `DetectionConfig.registry_value`), parse with `Version::parse()`. Return `Unavailable` on non-Windows via `cfg(not(windows))` stub.
- [ ] T011 [US1] Handle registry edge cases in `crates/astro-up-core/src/detect/registry.rs` — empty `DisplayVersion` returns `InstalledUnknownVersion`, missing key returns `NotInstalled`, permission denied returns `Unavailable` with diagnostic
- [ ] T012 [US1] Integration test for registry detection in `crates/astro-up-core/tests/detect_registry.rs` — test with a mock registry key on Windows CI, test non-Windows stub returns `Unavailable`. Use `insta` snapshot for result serialization.
- [ ] T013 [US1] Add `Serialize`/`Deserialize` derives to `DetectionResult` and related types for snapshot testing in `crates/astro-up-core/src/detect/mod.rs`

**Checkpoint**: Registry detection works — finds packages via uninstall keys with version extraction

---

## Phase 4: User Story 2 — Detect Version from PE File (Priority: P2)

**Goal**: Extract version from PE file headers as fallback when registry detection fails

**Independent Test**: Point PE detection at the test fixture, verify it extracts version 3.2.1.0

- [ ] T014 [P] [US2] Implement `pe::detect()` in `crates/astro-up-core/src/detect/pe.rs` — use `pelite::FileMap::open(path)` → `PeFile::from_bytes()` → `resources()` → `version_info()` → `fixed()` for `VS_FIXEDFILEINFO.dwFileVersion`. Fall back to string `"FileVersion"` if `fixed()` returns `None`. Resolve path via `PathResolver` first. Wrap in `spawn_blocking` for async compat.
- [ ] T015 [P] [US2] Implement `file::detect_exists()` and `file::detect_config()` in `crates/astro-up-core/src/detect/file.rs` — `file_exists` checks path presence (returns `InstalledUnknownVersion` or `NotInstalled`), `config_file` reads version from a text/JSON/TOML config file using `DetectionConfig.version_regex`
- [ ] T016 [US2] Integration test for PE detection using test fixture in `crates/astro-up-core/tests/detect_pe.rs` — verify version 3.2.1.0 extracted from `tests/fixtures/test.exe`, test missing file returns `NotInstalled`, test PE without version resource returns `InstalledUnknownVersion`
- [ ] T017 [US2] Integration test for full chain (registry → PE fallback) in `crates/astro-up-core/tests/detect_chain.rs` — test that PE is tried when registry returns `NotInstalled`, test that chain stops when registry succeeds

**Checkpoint**: PE fallback works — chain correctly falls through from registry to PE to file_exists

---

## Phase 5: User Story 3 — Detect Drivers via WMI (Priority: P3)

**Goal**: Detect driver versions via WMI `Win32_PnPSignedDriver` queries with 10s timeout

**Independent Test**: Query WMI for a known driver provider, verify version is returned

- [ ] T018 [US3] Implement `wmi_driver::detect()` in `crates/astro-up-core/src/detect/wmi_driver.rs` — create `WMIConnection` (via `spawn_blocking`), define `Win32PnPSignedDriver` serde struct with `DriverProviderName`, `DeviceClass`, `InfName`, `DriverVersion`, `DeviceID` fields. Execute `async_raw_query` with WHERE clause from `DetectionConfig` fields (`inf_provider`, `device_class`, `inf_name`). Filter with AND logic. Wrap in `tokio::time::timeout(Duration::from_secs(10))`.
- [ ] T019 [US3] Handle WMI edge cases in `crates/astro-up-core/src/detect/wmi_driver.rs` — timeout returns `Unavailable("WMI query timed out")`, WMI service unavailable returns `Unavailable` with COM error, no matching driver returns `NotInstalled`, multiple matches returns first match version (manifest `device_class` filters to relevant device)
- [ ] T020 [US3] Windows-only integration test for WMI in `crates/astro-up-core/tests/detect_wmi.rs` — `#[cfg(windows)]` test querying `Win32_PnPSignedDriver` for any driver, test non-Windows stub. Use `insta` snapshot.

**Checkpoint**: WMI driver detection works — queries return driver versions with timeout enforcement

---

## Phase 6: User Story 4 — Brownfield Hardware Discovery via VID:PID (Priority: P4)

**Goal**: Scan connected USB devices, match VID:PID against manifests, suggest unmanaged packages

**Independent Test**: Parse known VID:PID patterns, verify wildcard matching works

- [ ] T021 [P] [US4] Implement `VidPid` type with parsing and matching in `crates/astro-up-core/src/detect/hardware.rs` — parse `"03C3:120A"` to `VidPid { vendor_id: 0x03C3, product_id: Some(0x120A) }`, parse `"03C3:*"` to wildcard. Implement `matches(&self, other: &VidPid) -> bool` with wildcard support.
- [ ] T022 [US4] Implement `discover_hardware()` in `crates/astro-up-core/src/detect/hardware.rs` — query `Win32_PnPEntity` via WMI, parse `DeviceID` for `USB\VID_xxxx&PID_xxxx` pattern, match against manifest `[hardware]` VID:PID entries, filter out already-managed packages (cross-reference with ledger). Return `Vec<HardwareMatch>`. Non-Windows stub returns empty vec.
- [ ] T023 [P] [US4] Unit tests for VID:PID parsing and matching in `crates/astro-up-core/src/detect/hardware.rs` (inline tests) — exact match, wildcard match, no match, invalid format
- [ ] T024 [US4] Windows-only integration test for hardware discovery in `crates/astro-up-core/tests/detect_hardware.rs` — query real `Win32_PnPEntity`, verify USB devices are enumerated (skip assertions on specific hardware)

**Checkpoint**: Hardware discovery works — VID:PID matching suggests relevant packages

---

## Phase 7: User Story 5 — ASCOM Profile Detection (Priority: P5)

**Goal**: Detect ASCOM drivers via ASCOM Profile registry keys

**Independent Test**: Query ASCOM Profile registry path, verify graceful handling when ASCOM Platform is absent

- [ ] T025 [US5] Implement `ascom::detect()` in `crates/astro-up-core/src/detect/ascom.rs` — read ASCOM Profile registry keys at `HKLM\SOFTWARE\ASCOM` (check Platform version >= 7), enumerate registered drivers under device type subkeys (Camera, Telescope, etc.), extract driver name and version. Non-Windows stub returns `Unavailable`.
- [ ] T026 [US5] Integration test for ASCOM detection in `crates/astro-up-core/tests/detect_ascom.rs` — test missing ASCOM Platform returns `NotInstalled` gracefully, test non-Windows stub

**Checkpoint**: ASCOM detection works — finds drivers via Profile registry or returns NotInstalled gracefully

---

## Phase 8: User Story 6 — Detection Result Caching (Priority: P6)

**Goal**: In-memory cache with event-driven invalidation avoids redundant scans

**Independent Test**: Run detection twice, verify second call uses cache; invalidate, verify re-scan

- [ ] T027 [US6] Implement `DetectionCache` in `crates/astro-up-core/src/detect/cache.rs` — `HashMap<PackageId, CacheEntry>` with `get()`, `insert()`, `invalidate(id)`, `invalidate_all()` methods. `CacheEntry` holds `DetectionResult` + `scanned_at` timestamp. Use `std::sync::RwLock` for thread safety.
- [ ] T028 [US6] Unit tests for cache operations in `crates/astro-up-core/src/detect/cache.rs` (inline tests) — insert/get, invalidate single, invalidate all, concurrent read access

**Checkpoint**: Cache works — lookup returns cached results, invalidation clears entries

---

## Phase 9: Scan Orchestration + Ledger Integration

**Purpose**: Wire everything together — full scan, ledger sync, public API

- [ ] T029 Implement `DetectionService` trait and `Scanner` struct in `crates/astro-up-core/src/detect/mod.rs` — `scan()` loads all packages via `CatalogReader::list_all()`, runs detection chain per package (with `DetectionConfig` from `Software.detection`), collects `PackageDetection` results and `ScanError`s, populates cache, returns `ScanResult`
- [ ] T030 Implement ledger sync in `crates/astro-up-core/src/detect/mod.rs` — first add `Acknowledged` variant to `LedgerSource` enum in `crates/astro-up-core/src/ledger.rs`. Then after scan completes, diff detected-installed set against existing `Acknowledged` ledger entries. Insert new, update changed versions, remove `Acknowledged` entries where scan returned `NotInstalled`. Do NOT remove `AstroUp`-sourced entries.
- [ ] T031 Handle packages without `DetectionConfig` in `crates/astro-up-core/src/detect/mod.rs` — packages with `Software.detection == None` are skipped (no detection possible without config). Log at debug level, do not report as error.
- [ ] T032 Integration test for full scan orchestration in `crates/astro-up-core/tests/detect_scan.rs` — test with mock catalog returning test packages, verify scan results include per-package outcomes and errors, verify ledger entries are created/removed. Use `insta` snapshot for `ScanResult`.
- [ ] T033 Integration test for ledger sync in `crates/astro-up-core/tests/detect_scan.rs` — test new detection creates `Acknowledged` entry, test gone detection removes `Acknowledged` entry, test `AstroUp` entries are preserved when package is uninstalled

**Checkpoint**: Full scan works end-to-end — catalog → detect → cache → ledger sync

---

## Phase 10: Polish & Cross-Cutting Concerns

- [ ] T034 [P] Add `#[cfg(windows)]` CI job for registry/WMI/ASCOM tests in `.github/workflows/ci.yml` — extend existing `check-windows` job to run detection integration tests
- [ ] T035 [P] Add `detect` module to `pub use` exports in `crates/astro-up-core/src/lib.rs` and verify public API surface with `cargo doc`
- [ ] T036 Verify SC-001 (full scan <5s) by adding a benchmark or timed integration test in `crates/astro-up-core/tests/detect_scan.rs`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1
- **Phases 3-4 (US1 Registry, US2 PE)**: Depend on Phase 2, can run in parallel
- **Phase 5 (US3 WMI)**: Depends on Phase 2, can run parallel with 3-4
- **Phase 6 (US4 Hardware)**: Depends on Phase 5 (reuses WMI connection pattern)
- **Phase 7 (US5 ASCOM)**: Depends on Phase 2, can run parallel with 3-5
- **Phase 8 (US6 Cache)**: Depends on Phase 2, can run parallel with 3-7
- **Phase 9 (Orchestration)**: Depends on ALL user story phases (3-8)
- **Phase 10 (Polish)**: Depends on Phase 9

### User Story Independence

- **US1 (Registry)**: Independent after Phase 2
- **US2 (PE)**: Independent after Phase 2
- **US3 (WMI)**: Independent after Phase 2
- **US4 (Hardware)**: Depends on US3 (WMI pattern reuse), depends on Phase 9 (ledger check for already-managed)
- **US5 (ASCOM)**: Independent after Phase 2
- **US6 (Cache)**: Independent after Phase 2

### Within Each User Story

- Types/models before service logic
- Implementation before integration tests
- Core logic before edge case handling

### Parallel Opportunities

**Phase 2** (after T004): T005, T006, T008, T009 can all run in parallel

**Phases 3-5, 7-8** can all start simultaneously after Phase 2:
```
Phase 2 done →  US1 (Registry)  ──┐
              →  US2 (PE)        ──┤
              →  US3 (WMI)       ──┼──→ Phase 9 (Orchestration) → Phase 10
              →  US4 (Hardware)* ──┤
              →  US5 (ASCOM)     ──┤
              →  US6 (Cache)     ──┘

* US4 depends on US3 for WMI patterns
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (types, chain runner, path resolver)
3. Complete Phase 3: US1 Registry detection
4. **STOP and VALIDATE**: Test registry detection independently
5. This alone covers ~80% of software detection

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (Registry) → MVP — detects most installed software
3. Add US2 (PE) → Fallback chain works — catches portable apps and self-updates
4. Add US3 (WMI) → Driver detection — covers USB serial and camera drivers
5. Add US5 (ASCOM) → ASCOM drivers detected
6. Add US4 (Hardware) → Brownfield discovery — suggests packages based on connected hardware
7. Add US6 (Cache) → Performance optimization — avoids redundant scans
8. Phase 9 (Orchestration) → Full scan wired up with ledger sync
9. Phase 10 (Polish) → CI, docs, performance validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Constitution requires integration tests (Principle III) — included per user story
- Windows-only tests gated with `#[cfg(windows)]` — run on existing `check-windows` CI job
- PE fixture (T003) must be a valid PE file — use a minimal compiled .exe or generate with a build script
- WMI connection creation is blocking — always use `spawn_blocking` for `WMIConnection::new()`
