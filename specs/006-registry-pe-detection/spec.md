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

When registry detection fails or isn't configured, the application falls back to reading the version from the PE file header (VS_FIXEDFILEINFO) of the executable.

**Why this priority**: PE detection is the universal fallback — every Windows executable has version info embedded.

**Independent Test**: Point detection at a known .exe file, verify it extracts the correct file version.

**Acceptance Scenarios**:

1. **Given** a manifest with `detection.fallback.method = "pe_file"` and `file_path = "{program_dir}/NINA/NINA.exe"`, **When** the primary registry detection fails, **Then** the PE file version is extracted
2. **Given** the PE file doesn't exist at the configured path, **When** fallback runs, **Then** it returns "not installed"
3. **Given** a PE file with version `3.1.2.1001`, **When** extracted, **Then** the version is parsed as semver `3.1.2` (4th component stripped)

---

### User Story 3 - Fallback Chain (Priority: P3)

A manifest configures a primary detection method and one or more fallbacks. The system tries each method in order until one succeeds or all fail.

**Why this priority**: Robust detection requires fallbacks — registry keys can be missing after portable installs.

**Independent Test**: Configure registry → PE fallback. Ensure PE is tried when registry fails.

**Acceptance Scenarios**:

1. **Given** a manifest with registry primary and PE fallback, **When** registry detection succeeds, **Then** the fallback is not attempted
2. **Given** a manifest with registry primary and PE fallback, **When** registry fails, **Then** PE detection is attempted
3. **Given** all detection methods fail, **When** the chain completes, **Then** "not installed" is returned

---

### User Story 4 - ASCOM Profile Detection (Priority: P4)

The application detects ASCOM drivers by reading the ASCOM Profile registry keys at `HKLM\SOFTWARE\ASCOM\{device_type}\{driver_id}`.

**Why this priority**: ASCOM drivers use a non-standard registry location that generic uninstall key scanning won't find.

**Independent Test**: With an ASCOM driver installed, verify detection finds it via the ASCOM Profile keys.

**Acceptance Scenarios**:

1. **Given** an ASCOM camera driver is registered, **When** detection runs with `method = "ascom_profile"`, **Then** the driver name and version are returned
2. **Given** no ASCOM Platform is installed, **When** ASCOM detection runs, **Then** it returns "not installed" gracefully

### Edge Cases

- What happens on non-Windows platforms (CI)? Registry detection returns "not available" with a clear message. PE detection via pelite works cross-platform.
- What happens when the registry value exists but is empty? Treat as "installed but version unknown."
- What happens with 32-bit vs 64-bit registry views? Check both WoW6432Node and native views.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST detect installed software by reading Windows uninstall registry keys (HKLM and HKCU)
- **FR-002**: System MUST extract version strings from configurable registry values (default: `DisplayVersion`)
- **FR-003**: System MUST detect software versions from PE file headers (VS_FIXEDFILEINFO)
- **FR-004**: System MUST support a fallback chain: try detection methods in order until one succeeds
- **FR-005**: System MUST detect ASCOM drivers via ASCOM Profile registry keys
- **FR-006**: System MUST expand path tokens (`{program_dir}`, `{config_dir}`) in detection config paths
- **FR-007**: System MUST parse extracted version strings into the Version type from spec 003
- **FR-008**: System MUST check both 32-bit and 64-bit registry views on 64-bit Windows
- **FR-009**: System MUST handle missing registry keys, missing files, and permission errors gracefully
- **FR-010**: System MUST support `file_exists` detection method (presence check without version)
- **FR-011**: System MUST support `config_file` detection method (regex extraction from config files)

### Key Entities

- **DetectionResult**: Enum of Installed(Version), NotInstalled, or Unavailable(reason)
- **DetectionChain**: Ordered list of detection methods to try for a given package
- **RegistrySource**: Registry hive, key path, and value name to read

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Detection scan completes for all ~95 packages in under 5 seconds
- **SC-002**: Registry detection correctly identifies installed versions for all test packages
- **SC-003**: PE file detection works identically on Windows and in Linux CI (via pelite)
- **SC-004**: Fallback chain tries all configured methods before reporting "not installed"

## Assumptions

- Windows is the primary detection platform; detection on macOS/Linux returns "unavailable" for registry/WMI methods
- PE detection via pelite works cross-platform (reads PE headers without executing)
- The ASCOM Platform must be installed for ASCOM Profile detection to work
- Depends on: spec 003 (Version type, DetectionConfig), spec 004 (path token expansion)
