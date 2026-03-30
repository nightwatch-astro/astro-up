# Research: 006-registry-pe-detection

**Date**: 2026-03-30

## Library Decisions

### pelite — PE file parsing
- **Decision**: pelite 0.10.0
- **Rationale**: Cross-platform (works on Linux/macOS CI via `FileMap` mmap), auto-detects PE32/PE64, provides `VS_FIXEDFILEINFO` and string version info. Already referenced in constitution.
- **API**: `PeFile::from_bytes()` → `resources()` → `version_info()` → `fixed()` for `VS_FIXEDFILEINFO`, `value(lang, key)` for string values
- **Key types**: `VS_FIXEDFILEINFO.dwFileVersion: VS_VERSION { Major, Minor, Patch, Build }`, `VersionInfo`, `Language`
- **Gotchas**: `fixed()` returns `None` if only string info present — fall back to string `"FileVersion"`. No async — use `spawn_blocking`. Multiple translation languages possible — handle empty array.
- **Alternatives**: `goblin` (generic binary parser, less PE-specific API), `object` (no version resource support)

### winreg — Windows registry access
- **Decision**: winreg 0.56
- **Rationale**: De facto standard for Rust registry access. Supports `KEY_WOW64_32KEY`/`KEY_WOW64_64KEY` via `open_subkey_with_flags()`. Already a transitive dependency via Tauri.
- **API**: `RegKey::predef(HKEY_LOCAL_MACHINE)` → `open_subkey_with_flags(path, KEY_READ | KEY_WOW64_64KEY)` → `get_value::<String>("DisplayVersion")`
- **Gotchas**: Windows-only, entire usage behind `cfg(windows)`. `open_subkey()` returns `io::Error` with `NotFound` for missing keys. HKLM read needs no elevation.
- **Alternatives**: Raw `windows-sys` bindings (too low-level), `registry` crate (unmaintained)

### wmi — WMI queries
- **Decision**: wmi 0.14
- **Rationale**: Typed async WMI queries via serde deserialization. Query `Win32_PnPSignedDriver` for driver detection AND `Win32_PnPEntity` for VID:PID hardware discovery — single crate covers both use cases.
- **API**: `WMIConnection::new()` → `async_raw_query("SELECT ... FROM Win32_PnPSignedDriver WHERE ...")` or `async_query::<T>()` with serde struct
- **Gotchas**: Windows-only, COM initialization required. Queries can hang — need tokio timeout wrapper. Connection creation is blocking — use `spawn_blocking`.
- **Alternatives**: Raw COM via `windows` crate (extremely verbose), PowerShell subprocess (slow, parsing fragile)

### USB device enumeration (VID:PID)
- **Decision**: Reuse `wmi` crate with `Win32_PnPEntity`
- **Rationale**: Already a dependency for driver detection. `DeviceID` field contains `USB\VID_xxxx&PID_xxxx` pattern. No additional crate needed.
- **Alternatives considered**: SetupAPI via `windows` crate (much more complex, unsafe FFI), `nusb`/`rusb` (overkill for VID:PID matching only), `sysinfo` (does NOT support USB enumeration)

### Path token expansion
- **Decision**: Custom utility in `astro-up-core`
- **Rationale**: Small scope (map token → platform directory). Use `directories` crate (already a dependency from spec 004) for platform paths, plus Windows-specific `env::var("ProgramFiles")` behind `cfg(windows)`.
- **Tokens**: `{program_files}`, `{program_files_x86}`, `{app_data}`, `{local_app_data}`, `{common_app_data}`, `{user_home}`

## Architecture Decisions

### Detection module layout
- **Decision**: `crates/astro-up-core/src/detect/` module with submodules per method
- **Layout**: `mod.rs` (public API, chain runner), `registry.rs`, `pe.rs`, `wmi_driver.rs`, `ascom.rs`, `file.rs`, `cache.rs`, `path.rs`, `hardware.rs`
- **Rationale**: Constitution Principle I (modules-first) — all in `astro-up-core`, no new crate

### Platform abstraction
- **Decision**: `cfg(windows)` on individual detection method modules, with cross-platform trait
- **Pattern**: `DetectionMethod` trait with `async fn detect(&self, config: &DetectionConfig) -> DetectionResult`. Windows implementations in `cfg(windows)` modules, stub implementations returning `Unavailable` on other platforms.
- **Rationale**: Constitution Principle II (Platform Awareness)

### Scan architecture
- **Decision**: Sequential per-package, parallel within methods where safe
- **Rationale**: Registry lookups are fast (~1ms each). WMI queries should batch (single query for all drivers). PE file reads can be parallelized. Total ~95 packages well within 5-second target.
- **Approach**: One WMI query for all `Win32_PnPSignedDriver` rows, cached. One WMI query for all `Win32_PnPEntity` rows (hardware). Registry sequential per-package. PE parallel via `spawn_blocking`.

### Ledger integration
- **Decision**: Detection scan writes `Acknowledged` entries, removes stale ones
- **Flow**: After scan completes, diff detected set vs current `Acknowledged` ledger entries. New detections → insert. Gone detections → remove. Changed versions → update.
- **Rationale**: Clarification from checklist review — scan is source of truth for externally-installed packages
