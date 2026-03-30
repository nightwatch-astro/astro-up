# Feature Specification: Release Pipeline

**Feature Branch**: `019-release-pipeline`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 018 — automated releases via Tauri bundler

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automated Version Bumps (Priority: P1)

A maintainer merges a PR with conventional commits. release-plz creates a release PR bumping Cargo.toml versions and updating CHANGELOG.md. When the release PR is merged, a GitHub Release is created with the version tag.

**Why this priority**: Automated versioning eliminates manual errors and drives the release pipeline.

**Independent Test**: Merge a `feat:` commit, verify release-plz creates a PR with minor bump + changelog.

**Acceptance Scenarios**:

1. **Given** a `feat:` commit merged to main, **When** release-plz runs, **Then** a PR is created bumping minor version with changelog
2. **Given** a `fix:` commit merged, **When** release-plz runs, **Then** patch version bump
3. **Given** the release PR is merged, **When** the release job runs, **Then** a GitHub Release is created with the version tag

---

### User Story 2 - Windows Installer Build (Priority: P2)

When a release is tagged, CI builds the Tauri NSIS installer for Windows. The installer includes both the GUI app and the CLI binary. The update bundle is signed with Ed25519 for the auto-updater.

**Why this priority**: The installer is the primary distribution artifact.

**Independent Test**: Trigger a release build, verify the NSIS installer is created with both GUI and CLI binaries.

**Acceptance Scenarios**:

1. **Given** a release tag is pushed, **When** CI runs, **Then** a Windows NSIS installer is built via Tauri bundler
2. **Given** the NSIS installer is built, **When** checking its contents, **Then** both `astro-up.exe` (GUI) and `astro-up-cli.exe` (CLI) are included
3. **Given** the build completes, **When** signing, **Then** the update bundle is signed with Ed25519
4. **Given** artifacts are ready, **When** publishing, **Then** installer + update bundle + standalone CLI .exe + Ed25519 signature are attached to the GitHub Release

---

### User Story 3 - Update Endpoint (Priority: P3)

A JSON file on GitHub Releases describes the latest version for tauri-plugin-updater. Running apps check this endpoint to discover new versions.

**Why this priority**: The self-update mechanism (spec 016) depends on this.

**Independent Test**: After a release, fetch the update endpoint JSON, verify it has correct version and download URL.

**Acceptance Scenarios**:

1. **Given** a new release, **When** the endpoint is published, **Then** it contains version, platform, URL, and Ed25519 signature
2. **Given** the app checks the endpoint, **When** a newer version exists, **Then** the app offers to update

---

### User Story 4 - Standalone CLI Release (Priority: P4)

The standalone CLI binary is attached to the same GitHub Release for users who want CLI-only (headless servers, scripting, no GUI needed).

**Why this priority**: Power users and CI/CD systems need the CLI without WebView2.

**Independent Test**: Download the standalone CLI .exe from the release, verify it runs without WebView2.

**Acceptance Scenarios**:

1. **Given** a release is published, **When** checking release assets, **Then** `astro-up-cli.exe` is listed separately from the installer
2. **Given** the CLI .exe is downloaded to a clean machine, **When** run, **Then** it works without WebView2 or Tauri runtime

---

### User Story 5 - crates.io Publish (Priority: P5)

When a release is tagged, the astro-up-core crate is published to crates.io for downstream consumers (nightwatch-esp32, other tools).

**Why this priority**: Other nightwatch-astro projects depend on astro-up-core types.

**Acceptance Scenarios**:

1. **Given** a release tag, **When** the publish job runs, **Then** astro-up-core is published to crates.io via OIDC trusted publishing
2. **Given** the CLI and GUI crates, **When** publishing, **Then** they are NOT published (application crates, not libraries)

### Edge Cases

- Build fails for a release tag: Release is created but without artifacts. Re-trigger manually.
- Ed25519 signing fails: Block the release. Unsigned update bundles must never be published.
- crates.io publish fails (e.g., version already exists): Non-blocking for the installer release. Log warning.
- Release-plz creates a PR but CI fails on the PR: Fix the issue before merging the release PR.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use release-plz for automated version bumps and changelog generation
- **FR-002**: System MUST build a Windows NSIS installer via Tauri bundler on release tags
- **FR-003**: System MUST bundle both GUI and CLI binaries in the NSIS installer
- **FR-004**: System MUST sign the update bundle with Ed25519 for tauri-plugin-updater
- **FR-005**: System MUST publish an update endpoint JSON on the GitHub Release
- **FR-006**: System MUST attach a standalone CLI binary to the GitHub Release
- **FR-007**: System MUST NOT publish any crates to crates.io — all crates are internal to the workspace
- **FR-009**: System MUST use the nightwatch-astro GitHub App token for cross-repo operations
- **FR-010**: System MUST reuse the shared rust-release workflow from nightwatch-astro/.github where applicable

### Release Artifacts

| Artifact | Description |
|----------|-------------|
| `astro-up-setup-{version}.exe` | NSIS installer (GUI + CLI) |
| `astro-up-cli-{version}.exe` | Standalone CLI binary |
| `astro-up-update-{version}.msi.zip` | Tauri update bundle (for auto-updater) |
| `astro-up-update-{version}.msi.zip.sig` | Ed25519 signature |
| `update.json` | Update endpoint for tauri-plugin-updater |

### Key Entities

- **ReleaseArtifact**: Installer, CLI binary, update bundle, signature
- **UpdateEndpoint**: JSON with version, platform, download URL, Ed25519 signature

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Release pipeline from tag to published artifacts completes in under 15 minutes
- **SC-002**: Update endpoint is available within 5 minutes of release
- **SC-003**: NSIS installer contains both GUI and CLI binaries

## Deferred

- **SignPath.io Authenticode signing** — deferred until tool is published. Without it, Windows SmartScreen warns on first install.
- **Scoop bucket update** — deferred. When implemented, auto-updates `astro-up.json` in scoop-bucket repo.

## Assumptions

- GitHub Actions Windows runner for Tauri NSIS build
- Ed25519 key pair stored as GitHub Actions secrets (separate from catalog minisign key)
- release-plz + shared workflow from nightwatch-astro/.github is the foundation
- Only astro-up-core is published to crates.io (library crate). CLI and GUI are application crates.
- Depends on: spec 018 (CI infrastructure), spec 016 (Tauri build config)
