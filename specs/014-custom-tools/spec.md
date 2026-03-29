# Feature Specification: Custom Tools

**Feature Branch**: `014-custom-tools`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 013 — add/remove user-defined tools from GitHub URLs

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Add Custom Tool from GitHub (Priority: P1)

A user discovers an astrophotography tool on GitHub that isn't in the official catalog. They run `astro-up add https://github.com/author/cool-tool` and the application fetches the latest release, lists Windows-compatible assets, and asks the user to pick one. A TOML manifest is generated and saved locally.

**Why this priority**: Users need to manage niche tools not covered by the official catalog.

**Independent Test**: Add a real GitHub repo URL, verify the release assets are listed and a manifest is generated.

**Acceptance Scenarios**:

1. **Given** a GitHub repo URL with releases, **When** `astro-up add <url>` runs, **Then** the latest release assets are fetched and Windows-compatible ones listed
2. **Given** the user selects an asset, **When** confirmed, **Then** a TOML manifest is generated with auto-detected install method and saved to `{data_dir}/astro-up/custom/`
3. **Given** a repo with no releases, **When** adding, **Then** a clear error is shown ("no releases found")

---

### User Story 2 - Auto-detect Install Method (Priority: P2)

The application infers the install method from the asset filename: `.msi` → MSI, `.exe` with "setup" → InnoSetup/NSIS, `.zip` → ZIP extraction, plain `.exe` → portable.

**Why this priority**: Reduces user friction — most tools can be auto-configured without manual TOML editing.

**Independent Test**: Present assets with different extensions, verify correct install method is inferred for each.

**Acceptance Scenarios**:

1. **Given** an asset named `CoolTool-Setup.exe`, **When** auto-detecting, **Then** install method is set to "exe" with silent switches
2. **Given** an asset named `CoolTool.msi`, **When** auto-detecting, **Then** install method is set to "msi"
3. **Given** an asset named `CoolTool-win64.zip`, **When** auto-detecting, **Then** install method is set to "zip"

---

### User Story 3 - Remove Custom Tool (Priority: P3)

A user runs `astro-up remove cool-tool` to remove a custom tool's manifest. The tool is unregistered from astro-up but not uninstalled from the system.

**Why this priority**: Users need to clean up custom tools they no longer want to track.

**Independent Test**: Add then remove a custom tool, verify the manifest is deleted.

**Acceptance Scenarios**:

1. **Given** a custom tool is registered, **When** `remove` runs, **Then** the manifest is deleted and the tool no longer appears in scans
2. **Given** a custom tool is registered, **When** removing, **Then** the tool is NOT uninstalled from the system (only the manifest is removed)

### Edge Cases

- GitHub URL is a repo without releases: Error "no releases found."
- All release assets are non-Windows (Linux, macOS): Error "no Windows-compatible assets found."
- Custom tool conflicts with an official catalog entry: Warn "this package ID already exists in the official catalog."
- Rate limit hit during GitHub API query: Use configured token (spec 004), report rate limit if still exceeded.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse GitHub repository URLs and fetch the latest release via GitHub API
- **FR-002**: System MUST filter release assets to Windows-compatible files (.exe, .msi, .zip)
- **FR-003**: System MUST auto-detect install method from asset filename patterns
- **FR-004**: System MUST generate a valid TOML manifest for the custom tool
- **FR-005**: System MUST save custom manifests to `{data_dir}/astro-up/custom/{package_id}.toml`
- **FR-006**: System MUST include custom tools in catalog operations (list, check, update, scan)
- **FR-007**: System MUST support removing custom tools by deleting their manifest
- **FR-008**: System MUST NOT uninstall the actual software when removing a custom tool
- **FR-009**: System MUST configure the checkver section to use the GitHub provider for update checking
- **FR-010**: System MUST warn if a custom tool ID conflicts with an official catalog entry

### Key Entities

- **CustomManifest**: A user-created TOML manifest stored locally, not in the official catalog
- **AssetCandidate**: A release asset with filename, URL, size, and inferred install method

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Adding a tool from a GitHub URL completes in under 10 seconds
- **SC-002**: Install method auto-detection is correct for 90%+ of common asset naming patterns
- **SC-003**: Custom tools appear alongside official catalog entries in all operations

## Assumptions

- Only GitHub repos are supported for custom tools in v1 (GitLab deferred)
- Custom tools are per-user (stored in user data dir, not shared)
- Depends on: spec 003 (types), spec 004 (config), spec 005 (catalog integration), spec 008 (GitHub provider)
