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

**Why this priority**: Silent install is the core value proposition — users don't click through wizards.

**Independent Test**: Download and silently install a test InnoSetup package. Verify it installs without user interaction.

**Acceptance Scenarios**:

1. **Given** an InnoSetup installer, **When** executed silently, **Then** it runs with `/VERYSILENT /NORESTART /SUPPRESSMSGBOXES` and exits with code 0
2. **Given** an MSI installer, **When** executed silently, **Then** it runs with `msiexec /i <file> /qn /norestart`
3. **Given** a ZIP package, **When** "installed", **Then** it extracts to the configured directory with zip-slip protection
4. **Given** a manifest with custom `[install.switches]`, **When** executing, **Then** the custom switches fully replace the defaults

---

### User Story 2 - Exit Code Interpretation (Priority: P2)

When an installer exits with a non-zero code, the application interprets it using the manifest's known exit codes table and reports a human-readable reason.

**Why this priority**: Raw exit codes are meaningless to users. Semantic interpretation enables actionable error messages.

**Independent Test**: Simulate exit code 3010, verify the system reports "Reboot required."

**Acceptance Scenarios**:

1. **Given** an installer exits with code 3010, **When** interpreted, **Then** the system reports "Reboot required" and marks install as successful-pending-reboot
2. **Given** an installer exits with code 1, **When** the manifest maps `1 = "package_in_use"`, **Then** "Package in use — close the application before updating" is reported
3. **Given** an unknown exit code, **When** interpreted, **Then** the raw code is reported with a generic error message

---

### User Story 3 - Admin Elevation (Priority: P3)

When an installer requires admin privileges, the system detects this and triggers elevation. In GUI mode, this means a UAC prompt. In CLI mode, it reports that elevation is needed.

**Why this priority**: Most astrophotography software requires admin installation.

**Independent Test**: Run an installer that requires elevation, verify UAC prompt appears (GUI) or elevation message is shown (CLI).

**Acceptance Scenarios**:

1. **Given** a manifest with `elevation = "required"`, **When** the installer starts, **Then** it runs with admin privileges
2. **Given** a manifest with `elevation = "self"`, **When** the installer starts, **Then** the installer handles its own elevation
3. **Given** an installer that fails with exit code 740, **When** detected, **Then** the system retries with elevation

---

### User Story 4 - Reboot Handling (Priority: P4)

When an installer requires a reboot (exit code 3010 or similar), the user is warned and offered the choice to reboot now or later. The application never reboots without explicit user consent.

**Why this priority**: Users may be mid-imaging session. Surprise reboots would be catastrophic.

**Independent Test**: Install a package that returns reboot-required code. Verify user gets a choice dialog, not an auto-reboot.

**Acceptance Scenarios**:

1. **Given** an installer exits with 3010 (reboot required), **When** in GUI mode, **Then** a dialog offers "Reboot Now" / "Reboot Later"
2. **Given** an installer exits with 3010, **When** in CLI mode, **Then** a message says "Reboot required to complete installation" and exit code indicates reboot-pending
3. **Given** the user chooses "Later", **When** the install completes, **Then** a persistent reminder is shown until reboot occurs

---

### User Story 5 - Uninstall (Priority: P5)

When `upgrade_behavior = "uninstall_previous"` is configured, or a user explicitly requests removal, the system finds the uninstall string from the registry and executes it silently. Only packages with a registered uninstaller are supported — portable/ZIP packages are removed by deleting their directory.

**Why this priority**: Some packages require uninstall before upgrade (e.g., driver packages that can't upgrade in-place).

**Independent Test**: Install a package, then uninstall it. Verify the uninstall command is extracted from registry and executed silently.

**Acceptance Scenarios**:

1. **Given** a package with `upgrade_behavior = "uninstall_previous"`, **When** upgrading, **Then** the current version is uninstalled before the new version is installed
2. **Given** a package with a registry uninstall string, **When** uninstalling, **Then** the uninstall command runs silently with appropriate switches
3. **Given** a ZIP/portable package, **When** uninstalling, **Then** the install directory is deleted (with confirmation)
4. **Given** a package with no uninstaller and no known install directory, **When** uninstalling, **Then** an error reports "uninstall not supported for this package"

---

### User Story 6 - Install Directory Override (Priority: P6)

A user configures a custom install directory. The system passes this to the installer using the appropriate switch for each type.

**Why this priority**: Power users want control over install locations.

**Independent Test**: Install with custom directory, verify software lands there.

**Acceptance Scenarios**:

1. **Given** a custom install directory, **When** an InnoSetup installer runs, **Then** `/DIR=<path>` is passed
2. **Given** a custom directory for MSI, **When** installing, **Then** `INSTALLDIR=<path>` is passed

### Edge Cases

- Installer hangs: Kill process after configurable timeout (default: 10 min, overridable per manifest `[install].timeout`).
- Installer requires reboot mid-install: Detect via exit code, present choice to user, never auto-reboot.
- ZIP archive contains zip-slip path (`../../../`): Reject the archive with a security error. Every path in the archive is validated.
- User cancels during install: Attempt to kill the installer process gracefully.
- DownloadOnly packages: No execution — download and open containing folder. User handles installation.
- Bootstrapper spawns child process and exits: Wait for entire process tree via Windows Job Objects.
- ZIP with single root directory: Extract contents directly (avoid double nesting like `target/NINA-3.1/NINA-3.1/`).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST execute installers silently using type-appropriate default switches
- **FR-002**: Manifest `[install.switches]` MUST fully replace (not merge with) default switches when present
- **FR-003**: System MUST interpret exit codes using per-manifest `known_exit_codes` mapping
- **FR-004**: System MUST map exit codes to semantic types (PackageInUse, RebootRequired, ElevationRequired, AlreadyInstalled, etc.)
- **FR-005**: System MUST support admin elevation (proactive from manifest, reactive from exit code 740)
- **FR-006**: System MUST present reboot-required as a user choice, never auto-reboot
- **FR-007**: System MUST support custom install directory via per-type switches (`/DIR=`, `INSTALLDIR=`, `/D=`)
- **FR-008**: System MUST extract ZIP packages with mandatory zip-slip protection
- **FR-009**: System MUST detect single-root-dir ZIPs and extract contents without double nesting
- **FR-010**: System MUST enforce configurable per-installer timeout (default: 10 min, overridable via manifest `[install].timeout`)
- **FR-011**: System MUST support pre_install and post_install command hooks from manifest
- **FR-012**: System MUST emit install events (started, progress, completed, failed, reboot_required)
- **FR-013**: System MUST support these installer types: exe, msi, innosetup, nullsoft, wix, burn, zip, zipwrap, portable, download_only
- **FR-014**: System MUST support cancellation — terminate installer process on user cancel
- **FR-015**: System MUST wait for the entire process tree (Job Objects) for bootstrapper-style installers
- **FR-016**: System MUST support uninstall for packages with a registered uninstall string in the registry
- **FR-017**: System MUST support uninstall for ZIP/portable packages by deleting the install directory (with confirmation)
- **FR-018**: System MUST support `upgrade_behavior = "uninstall_previous"` — uninstall current before installing new
- **FR-019**: Installer type MUST be explicitly specified in the manifest — no auto-detection
- **FR-020**: System MUST record successful installs in the install ledger (spec 003 LedgerEntry) with path and version

### Default Silent Switches

| Installer Type | Default Switches |
|---------------|-----------------|
| InnoSetup | `/VERYSILENT /NORESTART /SUPPRESSMSGBOXES` |
| MSI | `msiexec /i <file> /qn /norestart` |
| NSIS (Nullsoft) | `/S` |
| WiX/Burn | `/quiet /norestart` |
| ZIP/ZipWrap | Extract to target directory |
| Portable | Copy to target directory |
| DownloadOnly | No execution — open containing folder |

### Key Entities

- **InstallRequest**: Package info, installer path, install directory, quiet flag, elevation requirement, timeout override
- **InstallResult**: Success, SuccessRebootRequired, Failed(SemanticError), Cancelled, Timeout
- **UninstallRequest**: Package info, uninstall command from registry or directory path
- **SemanticExitCode**: PackageInUse, RebootRequired, ElevationRequired, AlreadyInstalled, MissingDependency, CancelledByUser, etc.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Silent installation works for all supported installer types in the test suite
- **SC-002**: Exit codes are correctly interpreted for 100% of known exit code mappings
- **SC-003**: ZIP extraction rejects all zip-slip attack patterns
- **SC-004**: Installer timeout prevents indefinite hangs
- **SC-005**: Uninstall succeeds for all packages with registered uninstall strings

## Assumptions

- Windows is the only platform for actual installation
- UAC prompts are handled by the OS — the app triggers elevation, Windows shows the prompt
- Pre-install hooks run at the same elevation as the installer; timeout 60 seconds
- Pre-install failure aborts the install; post-install failure warns but doesn't fail
- Installer type is always specified in the manifest — never auto-detected
- WebView2 bootstrapping is the Tauri installer's job (spec 019), not this spec
- Depends on: spec 003 (types, LedgerEntry), spec 004 (config for timeouts/paths), spec 010 (download provides the installer file)
