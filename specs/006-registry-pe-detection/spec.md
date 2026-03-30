# Feature Specification: Windows Registry and PE Detection

**Feature Branch**: `006-registry-pe-detection`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 005 — detect installed software via Windows registry and PE file version info

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detect Installed Software via Registry (Priority: P1)

A user runs a scan and the application detects all installed astrophotography software by querying Windows uninstall registry keys. For each detected package, it reads the installed version from the registry value specified in the manifest's detection config.

**Why this priority**: Registry detection covers ~80% of installed software (EXE, MSI, InnoSetup installers all register uninstall keys).

**Independent Test**: Install NINA, run detection, verify it finds NINA with the correct version string.

**Acceptance Scenarios**:

1. **Given** NINA is installed with registry key `SOFTWARE\NINA`, **When** detection runs, **Then** the installed version is extracted from `DisplayVersion`
2. **Given** software is installed for current user only (HKCU), **When** detection runs, **Then** it checks both HKLM and HKCU uninstall keys
3. **Given** software is not installed, **When** detection runs for that package, **Then** it returns "not installed" without error

---

### User Story 2 - Detect Version from PE File (Priority: P2)

When registry detection fails or isn't configured, the application falls back to reading the version from the PE file header (VS_FIXEDFILEINFO) of the executable. The file path comes from either the manifest config or the install ledger (for packages installed by astro-up).

**Why this priority**: PE detection is the universal fallback — every Windows executable has version info embedded. It also catches self-updates (app updates itself between astro-up scans).

**Independent Test**: Point detection at a known .exe file, verify it extracts the correct file version.

**Acceptance Scenarios**:

1. **Given** a manifest with `detection.fallback.method = "pe_file"` and a configured path, **When** the primary registry detection fails, **Then** the PE file version is extracted
2. **Given** a package installed by astro-up (path in ledger) but no registry entry (portable app), **When** detection runs, **Then** PE detection uses the ledger path to find the executable
3. **Given** a portable app that self-updated from 3.1.0 to 3.2.0, **When** PE detection runs on the ledger path, **Then** the current version 3.2.0 is returned (not the version astro-up installed)
4. **Given** the PE file doesn't exist at the configured path, **When** fallback runs, **Then** it returns "not installed"

---

### User Story 3 - Fallback Chain (Priority: P3)

The default detection chain is: registry → pe_file → file_exists. The system tries each method in order until one succeeds or all fail. Manifests can override the chain with an explicit `[detection]` section.

**Why this priority**: Robust detection requires fallbacks — registry keys can be missing after portable installs.

**Independent Test**: Configure registry → PE fallback. Ensure PE is tried when registry fails.

**Acceptance Scenarios**:

1. **Given** a manifest with no explicit `[detection]` section, **When** detection runs, **Then** the default chain (registry → pe_file → file_exists) is used
2. **Given** a manifest with explicit `[detection]` overriding the chain, **When** detection runs, **Then** only the configured methods are tried in the specified order
3. **Given** all detection methods fail, **When** the chain completes, **Then** "not installed" is returned

---

### User Story 4 - ASCOM Profile Detection (Priority: P4)

The application detects ASCOM drivers by reading the ASCOM Profile registry keys. ASCOM Platform 7+ is the minimum supported version.

**Why this priority**: ASCOM drivers use a non-standard registry location that generic uninstall key scanning won't find. Many devices still use COM drivers even with ASCOM Platform 7.

**Independent Test**: With an ASCOM driver installed, verify detection finds it via the ASCOM Profile keys.

**Acceptance Scenarios**:

1. **Given** an ASCOM camera driver is registered, **When** detection runs with `method = "ascom_profile"`, **Then** the driver name and version are returned
2. **Given** no ASCOM Platform is installed, **When** ASCOM detection runs, **Then** it returns "not installed" gracefully
3. **Given** ASCOM Platform 7 with registered drivers, **When** detection runs, **Then** all registered drivers are found via the profile registry

---

### User Story 5 - Detection Result Caching (Priority: P5)

Detection results are cached in memory with event-driven invalidation. When astro-up installs or updates a package, that package's cached result is invalidated. An explicit `scan` command invalidates the entire cache. This prevents redundant scans during a single session (dashboard refresh, pre-update check, post-install verify).

**Why this priority**: Avoids redundant registry/PE scans during a session while staying accurate for astro-up's own operations.

**Independent Test**: Run detection twice in a session — second call returns cached result. Install a package — its cache entry is invalidated.

**Acceptance Scenarios**:

1. **Given** a detection scan was run 10 seconds ago, **When** the dashboard refreshes, **Then** cached results are returned without re-scanning
2. **Given** astro-up installs a package, **When** post-install verification runs, **Then** that package is re-detected (cache invalidated) but others use cache
3. **Given** the user runs `astro-up scan`, **When** the scan starts, **Then** the entire cache is cleared and all packages are re-detected

### Edge Cases

- Non-Windows platforms (CI): Registry detection returns "unavailable." PE detection via pelite works cross-platform.
- Registry value exists but is empty: Treat as "installed but version unknown."
- 32-bit vs 64-bit registry: Check both WOW6432Node and native views on 64-bit Windows.
- PE file path from ledger but file was moved/deleted: Return "not installed" — the app was uninstalled outside astro-up.
- External install/uninstall (user does it manually): Detected on next explicit `scan` (cache cleared). Not detected between scans.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect installed software by reading Windows uninstall registry keys (HKLM and HKCU)
- **FR-002**: System MUST extract version strings from configurable registry values (default: `DisplayVersion`)
- **FR-003**: System MUST detect software versions from PE file headers (VS_FIXEDFILEINFO)
- **FR-004**: System MUST support a default detection chain: registry → pe_file → file_exists
- **FR-005**: System MUST allow manifests to override the default chain with explicit `[detection]` config
- **FR-006**: System MUST detect ASCOM drivers via ASCOM Profile registry keys (ASCOM Platform 7+ minimum)
- **FR-007**: System MUST resolve PE file paths from the manifest config OR the install ledger (for astro-up-installed packages)
- **FR-008**: System MUST parse extracted version strings into the Version type from spec 003 (strip 4th component, pad missing)
- **FR-009**: System MUST check both 32-bit and 64-bit registry views on 64-bit Windows
- **FR-010**: System MUST handle missing registry keys, missing files, and permission errors gracefully
- **FR-011**: System MUST support `file_exists` detection method (presence check without version)
- **FR-012**: System MUST support `config_file` detection method (regex extraction from config files)
- **FR-013**: System MUST cache detection results in memory with event-driven invalidation (per-package on install/update, full on explicit scan)
- **FR-014**: System MUST compile on all platforms — Windows-only APIs behind `cfg(windows)`, returns Unavailable on other platforms

### Key Entities

- **DetectionResult**: Enum of Installed(Version), InstalledUnknownVersion, NotInstalled, or Unavailable(reason)
- **DetectionChain**: Ordered list of detection methods. Default: registry → pe_file → file_exists. Overridable per manifest.
- **DetectionCache**: In-memory cache of DetectionResult per package ID. Invalidated per-package by install/update events, fully by scan command.
- **PathResolver**: Resolves PE file paths from manifest config tokens OR install ledger entries.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Full detection scan completes for all ~95 packages in under 5 seconds
- **SC-002**: Cached detection lookups return in under 1ms
- **SC-003**: PE file detection works identically on Windows and in Linux CI (via pelite)
- **SC-004**: Self-updated apps (version changed outside astro-up) are detected correctly via PE on next scan

## Assumptions

- Windows is the primary detection platform; macOS/Linux returns Unavailable for registry/WMI methods
- PE detection via pelite works cross-platform (reads PE headers without executing)
- ASCOM Platform 7 is the minimum supported version (maintains profile registry for backward compat)
- The install ledger (spec 003 LedgerEntry) provides paths for astro-up-installed packages
- Detection cache is per-session (in-memory), not persisted to disk
- Depends on: spec 003 (Version type, DetectionConfig, LedgerEntry), spec 004 (path token expansion)
