# Feature Specification: ASCOM Alpaca Client

**Feature Branch**: `009-alpaca-client`
**Created**: 2026-03-29
**Status**: Deferred
**Deferred Reason**: Alpaca is device control, not driver management. Not needed for astro-up
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 008 — query ASCOM devices via Alpaca HTTP/JSON API

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Discover ASCOM Devices (Priority: P1)

A user has an ASCOM-compatible mount, camera, or focuser connected. The application discovers the ASCOM Alpaca server running on the local network (port 11111) and lists all registered devices with their types, names, and driver versions.

**Why this priority**: Discovery is the foundation — without finding the Alpaca server, no device queries are possible.

**Independent Test**: With ASCOM Platform running, discover the Alpaca server and list connected devices.

**Acceptance Scenarios**:

1. **Given** ASCOM Platform is running with Alpaca enabled, **When** discovery runs, **Then** the Alpaca server is found and its API version is reported
2. **Given** no ASCOM Platform is running, **When** discovery runs, **Then** it times out gracefully and reports "no Alpaca server found"
3. **Given** multiple ASCOM devices are registered, **When** querying the server, **Then** all device types and instances are listed

---

### User Story 2 - Query Device Driver Version (Priority: P2)

The application queries a specific ASCOM device (e.g., camera at device number 0) to retrieve its driver version and capabilities. This version is compared against the catalog to determine if a driver update is available.

**Why this priority**: Driver version detection via Alpaca is more reliable than registry scanning for ASCOM devices.

**Independent Test**: Query a connected ASCOM camera, verify the driver name and version are returned.

**Acceptance Scenarios**:

1. **Given** a connected ASCOM camera, **When** querying its driver info, **Then** the driver name, version, and interface version are returned
2. **Given** a device that is registered but not connected, **When** querying, **Then** a "not connected" status is returned

---

### User Story 3 - Fallback to Registry Detection (Priority: P3)

When Alpaca is unavailable (ASCOM Platform not installed, or server not running), the application falls back to detecting ASCOM drivers via registry keys (spec 006).

**Why this priority**: Not all users have Alpaca enabled. Registry fallback ensures detection still works.

**Independent Test**: With Alpaca unavailable, verify ASCOM drivers are detected via registry.

**Acceptance Scenarios**:

1. **Given** Alpaca is unavailable, **When** ASCOM detection runs, **Then** it falls back to ASCOM Profile registry detection
2. **Given** Alpaca returns a version AND registry has a version, **When** both are available, **Then** the Alpaca version takes precedence (more authoritative)

### Edge Cases

- Alpaca server is running but returns errors for specific device types: Report per-device errors, don't fail the entire discovery.
- ASCOM Platform is installed but Alpaca is not enabled: Discovery times out, falls back to registry.
- Multiple Alpaca servers on the network: Use the first discovered (mDNS) or the configured host.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST discover ASCOM Alpaca servers on the local network
- **FR-002**: System MUST query the Alpaca management API to list configured devices
- **FR-003**: System MUST query individual device endpoints for driver name and version
- **FR-004**: System MUST use the ascom-alpaca-core types for API communication
- **FR-005**: System MUST fall back to ASCOM Profile registry detection when Alpaca is unavailable
- **FR-006**: System MUST support configurable Alpaca server host and port (default: localhost:11111)
- **FR-007**: System MUST handle connection timeouts gracefully (default: 5 seconds)
- **FR-008**: System MUST support discovery via mDNS (Alpaca standard) or direct host configuration

### Key Entities

- **AlpacaServer**: Host, port, API version, list of configured devices
- **AlpacaDevice**: Device type, device number, driver name, driver version, connected status
- **DiscoveryResult**: Found(AlpacaServer) or NotFound(fallback strategy)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Alpaca discovery completes in under 5 seconds
- **SC-002**: Device queries return correct driver versions for all connected ASCOM devices
- **SC-003**: Fallback to registry works transparently when Alpaca is unavailable

## Assumptions

- ASCOM Alpaca specification is followed by all ASCOM Platform installations (v6.6+)
- The ascom-alpaca-core crate (nightwatch-astro/ascom-alpaca-core) provides the API types
- mDNS discovery may not work in all network configurations (firewalls, VLANs)
- Depends on: spec 003 (types), spec 006 (registry fallback)

## Clarifications

- **Discovery protocol**: Alpaca standard uses UDP broadcast on port 32227 for discovery. The response contains the API server's host:port. Fallback: configured static host:port in config (spec 004).
- **API versioning**: Query `/management/apiversions` first to confirm API compatibility. Require Alpaca API v1 minimum.
- **Device enumeration**: `/management/v1/configureddevices` returns all registered devices. Each has deviceType, deviceNumber, deviceName.
- **Driver version query**: Per-device `/api/v1/{deviceType}/{deviceNumber}/driverversion` returns the version string. Also query `/interfaceversion` for the ASCOM interface version.
- **Connection state**: Devices can be configured but not connected. Query `/connected` before assuming the device is usable. Report "configured but not connected" vs "not found".
- **Timeout strategy**: 2s for discovery broadcast, 5s for API calls. These are LAN operations — anything longer indicates a problem.
- **Alpaca vs registry precedence**: If both Alpaca and registry return a version, Alpaca wins (it reports the actually loaded driver, not what's registered).
