# Feature Specification: Installer Execution

**Feature Branch**: `011-installer-execution`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 010 — execute silent installers with exit code interpretation

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Silent Installation (Priority: P1)

A user selects a software update and the application executes the downloaded installer silently (no GUI prompts). The installer type determines the default silent switches: InnoSetup uses `/VERYSILENT /NORESTART`, MSI uses `/qn /norestart`, NSIS uses `/S`.

**Why this priority**: Silent install is the core value proposition — users don't have to click through installer wizards.

**Independent Test**: Download and silently install a test InnoSetup package. Verify it installs without user interaction and the expected files exist.

**Acceptance Scenarios**:

1. **Given** an InnoSetup installer, **When** executed silently, **Then** it runs with `/VERYSILENT /NORESTART /SUPPRESSMSGBOXES` and exits with code 0
2. **Given** an MSI installer, **When** executed silently, **Then** it runs with `msiexec /i <file> /qn /norestart` and exits with code 0
3. **Given** a ZIP package, **When** "installed", **Then** it extracts to the configured directory with zip-slip protection

---

### User Story 2 - Exit Code Interpretation (Priority: P2)

When an installer exits with a non-zero code, the application interprets it using the manifest's known exit codes table and reports a human-readable reason (e.g., "Package in use — close NINA before updating").

**Why this priority**: Raw exit codes are meaningless to users. Semantic interpretation enables actionable error messages.

**Independent Test**: Simulate exit code 3010 from an InnoSetup installer, verify the system reports "Reboot required."

**Acceptance Scenarios**:

1. **Given** an installer exits with code 3010, **When** interpreted, **Then** the system reports "Reboot required" and marks the install as successful-with-reboot
2. **Given** an installer exits with code 1, **When** the manifest maps `1 = "package_in_use"`, **Then** the system reports "Package in use"
3. **Given** an unknown exit code, **When** interpreted, **Then** the system reports the raw code with a "contact support" suggestion

---

### User Story 3 - Admin Elevation (Priority: P3)

When an installer requires admin privileges, the system detects this (either proactively from the manifest or reactively from a permission error) and prompts for elevation.

**Why this priority**: Most astrophotography software requires admin installation. Users expect a UAC prompt, not a cryptic error.

**Independent Test**: Run an installer that requires elevation. Verify it triggers a UAC prompt (or reports the need for elevation in CLI mode).

**Acceptance Scenarios**:

1. **Given** a manifest with `elevation = "required"`, **When** the installer starts, **Then** it runs with admin privileges (UAC prompt)
2. **Given** a manifest with `elevation = "self"`, **When** the installer starts, **Then** the installer handles its own elevation
3. **Given** an installer that fails with exit code 740 (elevation required), **When** detected, **Then** the system retries with elevation

---

### User Story 4 - Install Directory Override (Priority: P4)

A user configures a custom install directory. The system passes this to the installer using the appropriate switch for each installer type.

**Why this priority**: Power users want control over install locations, especially on multi-drive setups.

**Independent Test**: Install with a custom directory, verify the software lands in the specified location.

**Acceptance Scenarios**:

1. **Given** a custom install directory `D:\Astro\NINA`, **When** an InnoSetup installer runs, **Then** `/DIR=D:\Astro\NINA` is passed
2. **Given** a custom directory for MSI, **When** installing, **Then** `INSTALLDIR=D:\Astro\NINA` is passed

### Edge Cases

- Installer hangs (no exit within timeout): Kill the process after configurable timeout (default: 10 minutes), report timeout error.
- Installer requires reboot mid-install: Detect via exit code, report to user, don't auto-reboot.
- ZIP archive contains a zip-slip path (../../../etc/passwd): Reject the archive with a security error.
- User cancels during install: Attempt to kill the installer process gracefully.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST execute installers silently using type-appropriate default switches
- **FR-002**: System MUST interpret installer exit codes using per-manifest known_exit_codes mapping
- **FR-003**: System MUST map exit codes to semantic error types (PackageInUse, RebootRequired, ElevationRequired, etc.)
- **FR-004**: System MUST support admin elevation (proactive from manifest config, reactive from exit code 740)
- **FR-005**: System MUST support custom install directory via installer-appropriate switches
- **FR-006**: System MUST extract ZIP packages with zip-slip protection (reject paths containing `..`)
- **FR-007**: System MUST enforce configurable per-installer timeout (default: 10 minutes)
- **FR-008**: System MUST support pre_install and post_install command hooks from the manifest
- **FR-009**: System MUST emit install events (started, progress, completed, failed)
- **FR-010**: System MUST support these installer types: exe, msi, innosetup, nullsoft, wix, burn, zip, zipwrap, portable
- **FR-011**: System MUST support cancellation — terminate installer process on user cancel

### Key Entities

- **InstallRequest**: Package info, installer path, install directory, quiet flag, elevation requirement
- **InstallResult**: Success, SuccessRebootRequired, Failed(SemanticError), Cancelled, Timeout
- **SemanticExitCode**: PackageInUse, RebootRequired, ElevationRequired, AlreadyInstalled, MissingDependency, etc.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Silent installation works for all supported installer types in the test suite
- **SC-002**: Exit codes are correctly interpreted for 100% of known exit code mappings
- **SC-003**: ZIP extraction rejects all zip-slip attack patterns
- **SC-004**: Installer timeout prevents indefinite hangs

## Assumptions

- Windows is the only platform for actual installation (detection works cross-platform but install doesn't)
- UAC prompts are expected and handled by the OS — the app triggers elevation, Windows shows the prompt
- Pre/post install hooks are simple shell commands, not arbitrary scripts (limited scope)
- Depends on: spec 003 (types), spec 004 (config for timeouts/paths), spec 010 (download manager provides the installer file)
