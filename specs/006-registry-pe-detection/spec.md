# Feature Specification: Software and Driver Detection

**Feature Branch**: `006-registry-pe-detection`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Specs 005+006 — detect installed software via registry, PE, WMI, ASCOM Profile; brownfield hardware-to-driver mapping via VID:PID

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

1. **Given** a manifest with PE fallback and a configured path, **When** registry detection fails, **Then** the PE file version is extracted
2. **Given** a package installed by astro-up (path in ledger) but no registry entry (portable app), **When** detection runs, **Then** PE detection uses the ledger path to find the executable
3. **Given** a portable app that self-updated from 3.1.0 to 3.2.0, **When** PE detection runs on the ledger path, **Then** the current version 3.2.0 is returned
4. **Given** the PE file doesn't exist at the configured path, **When** fallback runs, **Then** it returns "not installed"

---

### User Story 3 - Detect Drivers via WMI (Priority: P3)

For driver packages (USB cameras, serial adapters, filter wheels), the application queries WMI `Win32_PnPSignedDriver` to detect installed driver versions. This is the only reliable method for USB serial drivers (FTDI, CP210x) that don't register uninstall keys.

**Why this priority**: Drivers are critical for astrophotography hardware. WMI is the only way to detect some driver types.

**Independent Test**: With a ZWO camera driver installed, run WMI detection with `inf_provider = "ZWO"`, verify the driver version is returned.

**Acceptance Scenarios**:

1. **Given** a ZWO camera driver is installed, **When** WMI detection runs with `inf_provider = "ZWO"`, **Then** the driver version is returned
2. **Given** a USB serial driver (FTDI) is installed, **When** WMI detection runs with `method = "driver_store"`, **Then** the driver version is returned
3. **Given** no matching driver is installed, **When** WMI detection runs, **Then** "not installed" is returned
4. **Given** WMI service is unavailable, **When** detection runs, **Then** it returns Unavailable with a diagnostic message

---

### User Story 4 - Brownfield Hardware Discovery via VID:PID (Priority: P4)

A user has astrophotography hardware connected but hasn't told astro-up about it. The application scans connected USB devices, matches VID:PID patterns against the manifest `[hardware]` section, and suggests relevant driver packages the user might want to track.

**Why this priority**: Brownfield onboarding — users switching to astro-up already have hardware and drivers. VID:PID matching bootstraps the relationship between hardware and packages.

**Independent Test**: With a ZWO camera connected (VID:PID `03C3:*`), run hardware discovery, verify it suggests the `zwo-asi-camera` package.

**Acceptance Scenarios**:

1. **Given** a USB device with VID:PID `03C3:120A` is connected, **When** hardware discovery runs, **Then** it matches to the ZWO ASI camera driver package
2. **Given** an unknown VID:PID, **When** hardware discovery runs, **Then** it is silently skipped (not an error)
3. **Given** the matched driver is already detected as installed (managed), **When** discovery runs, **Then** it's not suggested again
4. **Given** wildcard matching `03C3:*`, **When** any ZWO device is connected, **Then** it matches the ZWO driver package

---

### User Story 5 - ASCOM Profile Detection (Priority: P5)

The application detects ASCOM drivers by reading the ASCOM Profile registry keys. ASCOM Platform 7+ is the minimum supported version.

**Why this priority**: ASCOM drivers use a non-standard registry location that generic uninstall key scanning won't find.

**Independent Test**: With an ASCOM driver installed, verify detection finds it via the ASCOM Profile keys.

**Acceptance Scenarios**:

1. **Given** an ASCOM camera driver is registered, **When** detection runs with `method = "ascom_profile"`, **Then** the driver name and version are returned
2. **Given** no ASCOM Platform is installed, **When** ASCOM detection runs, **Then** it returns "not installed" gracefully

---

### User Story 6 - Detection Result Caching (Priority: P6)

Detection results are cached in memory with event-driven invalidation. When astro-up installs or updates a package, that package's cached result is invalidated. An explicit `scan` command invalidates the entire cache.

**Why this priority**: Avoids redundant registry/PE/WMI scans during a session.

**Independent Test**: Run detection twice — second call returns cached result. Install a package — its cache entry is invalidated.

**Acceptance Scenarios**:

1. **Given** a detection scan was run 10 seconds ago, **When** the dashboard refreshes, **Then** cached results are returned
2. **Given** astro-up installs a package, **When** post-install verification runs, **Then** that package is re-detected (cache invalidated)
3. **Given** the user runs `astro-up scan`, **When** the scan starts, **Then** the entire cache is cleared

### Edge Cases

- Non-Windows platforms (CI): Registry and WMI return "unavailable." PE detection via pelite works cross-platform.
- Registry value exists but is empty: Treat as "installed but version unknown."
- 32-bit vs 64-bit registry: Check both WOW6432Node and native views.
- PE file path from ledger but file was moved/deleted: Return "not installed."
- External install/uninstall: Detected on next explicit `scan` (cache cleared). Externally uninstalled packages (`Acknowledged` ledger entries where scan returns NotInstalled) are removed from the ledger — scan is the source of truth.
- Multiple WMI matches for same vendor (camera + EFW + focuser): Return all matches, manifest's `device_class` determines relevance.
- WMI timeout: Default 10 seconds. On timeout, return Unavailable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect installed software by scanning ALL catalog packages via Windows uninstall registry keys (HKLM and HKCU)
- **FR-002**: System MUST extract version strings from configurable registry values (default: `DisplayVersion`)
- **FR-003**: System MUST detect software versions from PE file headers (VS_FIXEDFILEINFO)
- **FR-004**: System MUST support a default detection chain: registry → pe_file → file_exists. Chain stops at the first method that returns Installed or InstalledUnknownVersion — remaining methods are not attempted.
- **FR-005**: System MUST allow manifests to override the default chain with explicit `[detection]` config
- **FR-006**: System MUST detect ASCOM drivers via ASCOM Profile registry keys (ASCOM Platform 7+ minimum)
- **FR-007**: System MUST resolve PE file paths from the manifest config OR the install ledger
- **FR-020**: System MUST provide a shared path token resolver that expands platform tokens (e.g., `{program_files}`, `{app_data}`, `{local_app_data}`) to actual filesystem paths
- **FR-008**: System MUST parse extracted version strings into the Version type from spec 003
- **FR-009**: System MUST check both 32-bit and 64-bit registry views on 64-bit Windows
- **FR-010**: System MUST handle missing registry keys, missing files, and permission errors gracefully — per-package errors MUST NOT abort the scan; successful results are returned alongside per-package error reports
- **FR-011**: System MUST support `file_exists` (checks path presence, returns InstalledUnknownVersion or NotInstalled) and `config_file` (reads file as text, applies `version_regex` capture group 1 as version string) detection methods
- **FR-012**: System MUST detect driver versions via WMI `Win32_PnPSignedDriver` queries
- **FR-013**: System MUST filter WMI results by `DriverProviderName`, `DeviceClass`, and `InfName`
- **FR-014**: System MUST match connected USB devices by VID:PID pattern against manifest `[hardware]` sections for brownfield discovery
- **FR-015**: System MUST support wildcard VID:PID matching (e.g., `03C3:*` for all products under a vendor ID)
- **FR-016**: System MUST cache detection results in memory with event-driven invalidation
- **FR-019**: System MUST persist detected installed packages as LedgerEntry records with `source = Acknowledged` — same update-tracking behavior as astro-up-installed packages, but distinguished by source for lifecycle operations (e.g., uninstall eligibility)
- **FR-017**: System MUST compile on all platforms — Windows-only APIs behind `cfg(windows)`
- **FR-018**: System MUST enforce a 10-second timeout on WMI queries

### Key Entities

- **DetectionResult**: Enum of Installed(Version), InstalledUnknownVersion, NotInstalled, or Unavailable(reason)
- **DetectionChain**: Ordered list of detection methods. Default: registry → pe_file → file_exists. Overridable per manifest.
- **DetectionCache**: In-memory cache of DetectionResult per package ID with event-driven invalidation.
- **PathResolver**: Resolves PE file paths from manifest config tokens OR install ledger entries.
- **WmiDriver**: Provider name, device class, INF name, driver version, device ID from Win32_PnPSignedDriver.
- **VidPidMatch**: Maps a VID:PID pattern from manifest `[hardware]` to a package ID. Used for brownfield discovery.
- **HardwareDiscovery**: Scans connected USB devices, matches VID:PID, suggests untracked packages.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Full detection scan (registry + PE + WMI) completes for all ~95 packages in under 5 seconds
- **SC-002**: Cached detection lookups return in under 1ms
- **SC-003**: PE file detection works identically on Windows and Linux (cross-platform)
- **SC-004**: VID:PID matching correctly identifies all known astrophotography hardware in the test set
- **SC-005**: WMI detection returns correct driver versions for USB serial and camera driver packages

## Clarifications

### Session 2026-03-30

- Q: Does detection scan all catalog packages or only user-tracked packages? → A: Scan all catalog packages. Any package detected as installed is automatically added as managed by astro-up. The user can then decide per package whether to update it or not. No manual opt-in to start tracking.
- Q: How should partial failures be handled when scanning ~95 packages? → A: Continue scan, report per-package errors alongside successful results. A single failure must not block the rest of the scan.
- Q: What happens when VID:PID discovery finds hardware for a package already detected as installed? → A: Skip suggesting it — it's already managed. Discovery only suggests packages where matching hardware is connected but the driver/software was NOT detected as installed.

## Assumptions

- Windows is the primary detection platform; macOS/Linux returns Unavailable for registry/WMI methods
- PE detection via pelite works cross-platform
- ASCOM Platform 7 is the minimum supported version
- The install ledger (spec 003 LedgerEntry) provides paths for astro-up-installed packages
- Detection cache is per-session (in-memory), not persisted to disk
- VID:PID matching is for brownfield discovery (suggesting packages), not for version detection. Device connection notifications (hotplug) are deferred.
- Depends on: spec 003 (Version type, DetectionConfig, LedgerEntry)
- Path token expansion (originally spec 004, dropped during SQLite pivot) will be built as a shared utility in astro-up-core within this spec — detection is the first consumer
