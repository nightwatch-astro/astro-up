# Feature Specification: WMI Driver Detection

**Feature Branch**: `007-wmi-driver-detection`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 006 — detect installed drivers via WMI queries

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Detect Installed Drivers (Priority: P1)

A user connects a ZWO camera and runs a scan. The application queries Windows WMI to find the installed driver, its version, and whether it matches the latest available version.

**Why this priority**: Drivers are critical for astrophotography hardware. Users need to know if their drivers are current.

**Independent Test**: With a ZWO camera driver installed, run detection, verify it returns the driver name and version.

**Acceptance Scenarios**:

1. **Given** a ZWO camera driver is installed, **When** WMI detection runs with `inf_provider = "ZWO"`, **Then** the driver version is returned
2. **Given** no matching driver is installed, **When** WMI detection runs, **Then** "not installed" is returned
3. **Given** multiple drivers from the same vendor, **When** detection runs, **Then** the most relevant match is returned (by device class)

---

### User Story 2 - Map USB Devices to Known Packages (Priority: P2)

When a USB device is connected, the application matches its VID:PID against known astrophotography hardware in the manifest database, identifying which driver package it needs.

**Why this priority**: Automatic hardware-to-driver mapping reduces user friction — they plug in a camera and see the relevant driver status.

**Independent Test**: Simulate a USB connect event with VID:PID 03C3:120A, verify it maps to "zwo-asi-camera-driver".

**Acceptance Scenarios**:

1. **Given** a USB device with VID:PID `03C3:120A`, **When** matched against manifests, **Then** it resolves to the ZWO ASI camera driver package
2. **Given** an unknown VID:PID, **When** matched, **Then** no package is returned (not an error)

### Edge Cases

- WMI service not running or inaccessible: Return "unavailable" with a diagnostic message.
- Multiple driver versions installed (side-by-side): Return the latest version.
- Running on non-Windows: Return "unavailable" — WMI is Windows-only.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST query Win32_PnPSignedDriver via WMI to detect installed drivers
- **FR-002**: System MUST filter drivers by provider name, device class, and INF name
- **FR-003**: System MUST match USB VID:PID patterns against manifest hardware configs
- **FR-004**: System MUST support wildcard VID:PID matching (e.g., `03C3:*` for all ZWO products)
- **FR-005**: System MUST handle WMI service unavailability gracefully
- **FR-006**: System MUST parse driver version strings from WMI into the Version type
- **FR-007**: System MUST support the `driver_store` detection method in DetectionConfig

### Key Entities

- **WmiDriver**: Provider name, device class, INF name, driver version, device ID
- **VidPidMapping**: Maps VID:PID patterns to package IDs from manifests

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: WMI driver detection completes in under 2 seconds per package
- **SC-002**: VID:PID matching correctly identifies all known astrophotography hardware in the test set
- **SC-003**: System degrades gracefully on non-Windows platforms

## Assumptions

- WMI is available on all supported Windows versions (10+)
- Driver manifests include `[hardware]` section with vid_pid, device_class, inf_provider
- Depends on: spec 003 (types), spec 006 (detection framework/chain)
