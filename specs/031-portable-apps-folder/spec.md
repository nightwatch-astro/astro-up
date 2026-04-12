# Feature Specification: Portable Apps Folder

**Feature Branch**: `031-portable-apps-folder`
**Created**: 2026-04-12
**Status**: Draft
**Type**: implementation
**Input**: User description: "Portable apps folder: configurable directory where non-installable apps (download_only, portable) are placed."

## Current Behavior

Two install methods are relevant:

| Method | Current behavior | Manifests | Handler |
|--------|-----------------|-----------|---------|
| `DownloadOnly` | Downloads file, opens Explorer on the download folder. No copy, no hooks, no elevation. Ledger records download dir. | 17 packages (firmware tools, NINA caches, utilities) | `handle_download_only` — early return, skips all install pipeline |
| `Portable` | Copies downloaded file to `resolve_install_dir()` (temp/download-derived path). Runs hooks, supports uninstall. | 0 packages (unused in practice) | `handle_portable_install` — copies file to target dir |

The key gap: `DownloadOnly` packages end up in a temporary downloads folder with no organization. Users must manually find and relocate them.

## User Scenarios & Testing

### User Story 1 — Install a download-only app to the portable apps folder (Priority: P1)

A user clicks "Install" on a `download_only` package (e.g., a firmware utility or standalone tool). Instead of just opening the download folder in Explorer, the app copies the downloaded file into the user's configured portable apps directory, organized by package name.

**Why this priority**: Core value proposition — 17 packages currently just dump files in a temp folder with no organization.

**Independent Test**: Install a `download_only` package and verify the file appears in the portable apps directory under a package-named subfolder.

**Acceptance Scenarios**:

1. **Given** a package with `method = "download_only"`, **When** the user clicks Install, **Then** the downloaded file is copied to `{portable_apps_dir}/{package-id}/` and the ledger records the install path.
2. **Given** the download is a zip archive, **When** the install completes, **Then** the archive is extracted (not just copied as a .zip) to the package subfolder.
3. **Given** an install completes to the portable folder, **When** the operation finishes, **Then** the UI shows the destination path (not "Open download folder").

---

### User Story 2 — Configure the portable apps directory (Priority: P2)

A user navigates to Settings > Paths and sets a custom directory for portable apps. This persists across sessions and is used for all future portable/download-only installs.

**Why this priority**: Users need control over where files go, especially if the default location doesn't suit them (e.g., limited disk space on C:, preference for a dedicated tools drive).

**Independent Test**: Change the portable apps directory in Settings, install a portable package, verify it lands in the new location.

**Acceptance Scenarios**:

1. **Given** the Settings > Paths section, **When** the user sets a custom portable apps directory, **Then** the setting persists and is used for the next portable install.
2. **Given** no custom directory is set, **When** a portable package is installed, **Then** the default directory is used (`{app_data}/nightwatch/astro-up/apps/`).
3. **Given** the configured directory does not exist, **When** a portable install starts, **Then** the directory is created automatically.

---

### User Story 3 — View and open the portable app location (Priority: P2)

After a portable app is installed, the user can see where it was placed and open the folder directly from the app's detail page.

**Why this priority**: Users need to find the installed portable app to create shortcuts, add it to PATH, or launch it manually.

**Independent Test**: Install a portable app, navigate to its detail page, verify the install path is visible and the "Open folder" action works.

**Acceptance Scenarios**:

1. **Given** a portable app is installed, **When** the user views the package detail, **Then** the install path is displayed in the Technical or Overview tab.
2. **Given** a portable app is installed, **When** the user clicks "Open folder" on the install path, **Then** the file explorer opens at that location.

---

### User Story 4 — Update replaces the app in-place (Priority: P3)

When a newer version of a download-only app is available and the user updates, the new version replaces the old one in the same portable folder location.

**Why this priority**: Without this, updates would create duplicate copies or leave old versions behind.

**Independent Test**: Install a download-only app, trigger an update, verify the old version is replaced and the ledger reflects the new version.

**Acceptance Scenarios**:

1. **Given** a download-only app is installed at `{portable_apps_dir}/{package-id}/`, **When** the user updates to a newer version, **Then** the old files are replaced with the new version in the same location.
2. **Given** an update is in progress, **When** the old version is replaced, **Then** the ledger is updated with the new version and the same install path.

---

### User Story 5 — Portable method uses the same folder (Priority: P3)

If any manifests are later changed to `method = "portable"`, the `handle_portable_install` handler should also use the configured portable apps directory instead of the current temp-derived path.

**Why this priority**: Currently 0 manifests use `portable`, but the method exists in the codebase. Aligning it with the new folder ensures consistency if adopted later.

**Independent Test**: Create a test manifest with `method = "portable"`, install it, verify it lands in the portable apps folder.

**Acceptance Scenarios**:

1. **Given** a package with `method = "portable"`, **When** installed, **Then** the file is copied to `{portable_apps_dir}/{package-id}/` (same as download-only).

---

### Edge Cases

- What happens when the portable apps directory is on a drive that doesn't exist or is full? Show a clear error message and suggest changing the directory in Settings.
- What happens when a portable app's download is a single `.exe` (not a zip)? Copy the exe directly into `{portable_apps_dir}/{package-id}/`.
- What happens when the user changes the portable directory after apps are already installed? Existing apps stay where they are; only new installs use the new directory. The ledger tracks actual paths.
- What happens when a portable app has a nested folder structure inside the zip? Extract preserving the structure into the package subfolder.
- Package IDs are enforced as `[a-z0-9-]` by the catalog schema — always valid as directory names.

## Requirements

### Functional Requirements

- **FR-001**: System MUST provide a configurable setting for the portable apps directory with a sensible default path.
- **FR-002**: The default portable apps directory MUST be under the user's profile directory (not a system-wide path requiring elevation).
- **FR-003**: When installing a `download_only` package, the system MUST copy the downloaded file to `{portable_apps_dir}/{package-id}/` instead of opening Explorer on the download folder.
- **FR-004**: When the downloaded file is a zip archive, the system MUST extract its contents to the package subfolder (not copy the zip as-is).
- **FR-004a**: The `handle_portable_install` handler MUST also use the configured portable apps directory as the install target.
- **FR-005**: After a portable install completes, the ledger MUST record the install path pointing to the portable apps directory.
- **FR-006**: The operation completion event MUST include the destination path so the UI can display it.
- **FR-007**: The portable apps directory MUST be created automatically if it does not exist.
- **FR-008**: The Settings > Paths section MUST include a field for configuring the portable apps directory.
- **FR-009**: When updating a portable app, the system MUST replace the old version in the same location.
- **FR-010**: The package detail view MUST show the install path for portable apps with an "Open folder" action.

### Key Entities

- **Portable Apps Directory**: A user-configurable path where non-installer packages are placed. Stored as a config setting. Defaults to a location under the user's app data directory.
- **Install Path**: Recorded in the ledger for each installed package. For portable apps, points to the package subfolder within the portable apps directory.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Portable/download-only packages are placed in an organized directory instead of left in the downloads folder.
- **SC-002**: Users can find installed portable apps without manually searching the downloads folder.
- **SC-003**: The portable apps directory setting is discoverable in Settings within 10 seconds.
- **SC-004**: Portable app install completes within 5 seconds of download finishing (copy/extract time).
- **SC-005**: 100% of portable installs record the correct path in the ledger.

## Assumptions

- The portable apps directory is per-user, not system-wide — no elevation is needed.
- `DownloadOnly` currently early-returns in the install pipeline (skips hooks, elevation, zip handling). This feature replaces `handle_download_only` with a proper copy/extract flow.
- `Portable` handler already copies files but targets a temp-derived path. This feature changes its target to the portable apps dir.
- Zip extraction uses the existing `zip` crate already in the codebase (used by `handle_zip_install`).
- The `install_path` field on `PackageComplete` events and ledger entries already exists — this feature populates it for portable/download-only packages.
- Windows is the only target platform; the default path uses Windows conventions (`AppData`).
- 17 manifests currently use `download_only`; 0 use `portable`. The primary impact is on `download_only`.
