# Feature Specification: Release Pipeline

**Feature Branch**: `019-release-pipeline`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 018 — automated releases via Tauri bundler

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automated Version Bumps (Priority: P1)

A maintainer merges a PR with conventional commits. release-plz automatically creates a release PR bumping the version in Cargo.toml and updating CHANGELOG.md based on commit history.

**Why this priority**: Automated versioning eliminates manual version management errors.

**Independent Test**: Merge a `feat:` commit, verify release-plz creates a PR with a minor version bump and changelog entry.

**Acceptance Scenarios**:

1. **Given** a `feat:` commit is merged, **When** release-plz runs, **Then** a PR is created bumping the minor version
2. **Given** a `fix:` commit is merged, **When** release-plz runs, **Then** a PR is created bumping the patch version
3. **Given** the release PR is merged, **When** the release job runs, **Then** a GitHub Release is created with the tag

---

### User Story 2 - Windows Installer Build (Priority: P2)

When a release is tagged, CI builds the Tauri NSIS installer for Windows, signs the update bundle with Ed25519, and attaches it to the GitHub Release.

**Why this priority**: The installer is the distribution artifact — without it, users can't install.

**Independent Test**: Trigger a release build, verify the NSIS installer is created and attached.

**Acceptance Scenarios**:

1. **Given** a release tag is pushed, **When** CI runs, **Then** a Windows NSIS installer is built via Tauri bundler
2. **Given** the build completes, **When** signing, **Then** the update bundle is signed with Ed25519 for the auto-updater
3. **Given** artifacts are ready, **When** publishing, **Then** installer + update bundle + signature are attached to the GitHub Release

---

### User Story 3 - Scoop Bucket Update (Priority: P3)

After a release, the Scoop bucket manifest is automatically updated with the new version, URL, and hash.

**Why this priority**: Scoop users get the update through their package manager.

**Independent Test**: Trigger a release, verify the scoop-bucket repo is updated with correct version and hash.

**Acceptance Scenarios**:

1. **Given** a new release, **When** the bucket update runs, **Then** `astro-up.json` in scoop-bucket is updated with the new version and SHA256
2. **Given** the bucket update PR is created, **When** CI passes, **Then** it auto-merges

---

### User Story 4 - Update Endpoint (Priority: P4)

A JSON file on GitHub Releases describes the latest version for tauri-plugin-updater. Running apps check this endpoint to discover new versions.

**Why this priority**: The self-update mechanism (spec 016) depends on this endpoint.

**Independent Test**: After a release, fetch the update endpoint JSON, verify it contains the correct version and download URL.

**Acceptance Scenarios**:

1. **Given** a new release, **When** the endpoint is published, **Then** it contains version, platform, URL, and Ed25519 signature
2. **Given** the app checks the endpoint, **When** a newer version exists, **Then** the app offers to update

### Edge Cases

- Build fails for the release tag: The release is created but without artifacts. Re-trigger the build manually.
- SignPath.io signing fails: Release proceeds without Authenticode (Ed25519 still applied). Log a warning.
- Scoop bucket update fails: Non-blocking — users can still download directly from GitHub Releases.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST use release-plz for automated version bumps and changelog generation
- **FR-002**: System MUST build Windows NSIS installer via Tauri bundler on release tags
- **FR-003**: System MUST sign update bundles with Ed25519 for tauri-plugin-updater
- **FR-004**: System MUST publish installer + update bundle + signature to GitHub Releases
- **FR-005**: System MUST generate a JSON update endpoint for tauri-plugin-updater
- **FR-006**: System MUST update the Scoop bucket manifest after each release
- **FR-007**: System MUST publish Rust crates to crates.io via trusted OIDC publishing
- **FR-008**: System MUST support SignPath.io Authenticode signing (deferred — conditional on SignPath account)
- **FR-009**: System MUST use GitHub App token (not PAT) for cross-repo operations (Scoop bucket update)

### Key Entities

- **ReleaseArtifact**: NSIS installer (.exe), update bundle (.msi.zip), Ed25519 signature (.sig)
- **UpdateEndpoint**: JSON describing latest version per platform, download URL, and signature
- **ScoopManifest**: JSON in scoop-bucket repo with version, URL, and hash

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Release pipeline from tag to published artifacts completes in under 15 minutes
- **SC-002**: Update endpoint is available within 5 minutes of release
- **SC-003**: Scoop bucket is updated within 10 minutes of release
- **SC-004**: crates.io publish succeeds without manual token management (OIDC)

## Assumptions

- GitHub Actions is the CI platform for release builds
- Windows runner required for Tauri NSIS build
- Ed25519 key pair stored as GitHub Actions secrets
- SignPath.io integration is deferred (issue #21 in old repo)
- Depends on: spec 018 (CI infrastructure), spec 016 (Tauri shell for build config)
