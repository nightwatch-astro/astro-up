# Feature Specification: Manifest Pipeline Modernization

**Feature Branch**: `020-manifest-modernization`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 019 — modernize nightwatch-astro/astro-up-manifests

## Note

This spec applies to the **nightwatch-astro/astro-up-manifests** repository, not the main astro-up repo. The main app's spec 005 (Catalog) consumes the artifacts produced by this pipeline.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - TOML to SQLite Compilation (Priority: P1)

A manifest maintainer edits a TOML manifest and pushes to main. CI compiles all TOMLs into a SQLite database (`catalog.db`), signs it with minisign, and publishes it as a GitHub Release asset.

**Why this priority**: SQLite is the new catalog format consumed by the Rust client (spec 005). The compilation pipeline is the critical path.

**Independent Test**: Modify a manifest TOML, push, verify the compiled catalog.db contains the change.

**Acceptance Scenarios**:

1. **Given** a TOML manifest change, **When** CI runs, **Then** catalog.db is recompiled with the updated data
2. **Given** the compilation succeeds, **When** signing, **Then** catalog.db.minisig is generated with the project's minisign key
3. **Given** signing succeeds, **When** publishing, **Then** catalog.db + signature are uploaded to a rolling `catalog/latest` GitHub Release

---

### User Story 2 - Automated Version Checking (Priority: P2)

CI runs the checker on a 6-hour schedule, querying vendor sites for new versions. Discovered versions are written to per-version files and the catalog is recompiled.

**Why this priority**: Automated checking is the core value of the manifest repo — keeping version data current without manual intervention.

**Independent Test**: Wait for the scheduled run, verify new version files are created for packages with actual updates.

**Acceptance Scenarios**:

1. **Given** the 6-hour schedule triggers, **When** the checker runs, **Then** all configured packages are checked for new versions
2. **Given** a new version is discovered, **When** writing, **Then** a version file is created at `versions/{package_id}/{semver}.json`
3. **Given** a check fails for one package, **When** the run completes, **Then** other packages are still checked (continue-on-error)

---

### User Story 3 - Self-Describing Manifests (Priority: P3)

Manifests use `[checkver]` instead of the old `[remote]` section, with Scoop-style `$version` template variables for constructing download URLs. The `[checkver]` section stays in the compiled catalog so the client can do its own version checking.

**Why this priority**: Self-describing manifests enable the client to check versions directly, not just rely on the CI checker.

**Independent Test**: Parse a manifest with `$version` in the autoupdate URL, verify the template resolves correctly.

**Acceptance Scenarios**:

1. **Given** a manifest with `autoupdate.url = "https://example.com/download/$version/setup.exe"`, **When** version `3.1.2` is discovered, **Then** the URL resolves to `https://example.com/download/3.1.2/setup.exe`
2. **Given** `$majorVersion`, **When** version is `3.1.2`, **Then** it resolves to `3`

---

### User Story 4 - Per-Version File Storage (Priority: P4)

Each discovered version is stored as an individual JSON file (`versions/{id}/{semver}.json`) instead of a monolithic `versions.json`. Git history serves as the audit trail.

**Why this priority**: Per-version files enable incremental updates and cleaner git history (one file per discovery, not one giant file with 95+ entries).

**Independent Test**: Check a package, verify a new version file is created. Check that catalog.db includes the version.

**Acceptance Scenarios**:

1. **Given** NINA 3.1.2 is discovered, **When** writing, **Then** `versions/nina-app/3.1.2.json` is created with url, sha256, discovered_at
2. **Given** the version file already exists, **When** re-discovered, **Then** it is not overwritten (idempotent)

### Edge Cases

- Checker discovers a version that's actually older than the latest: Write it but don't update the "latest" pointer. Historical versions are still valid.
- Minisign signing fails: Block the release. An unsigned catalog must never be published.
- GitHub Release asset upload fails: Retry once. If still failing, alert via GitHub issue.
- Manifest TOML has a validation error: Reject at compile time with the filename and error. Don't publish a broken catalog.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST compile TOML manifests into a SQLite catalog database
- **FR-002**: System MUST sign catalog.db with minisign before publishing
- **FR-003**: System MUST publish catalog.db + signature to GitHub Releases as a rolling latest tag
- **FR-004**: System MUST run version checking on a 6-hour cron schedule
- **FR-005**: System MUST store discovered versions as per-version JSON files
- **FR-006**: System MUST support `$version`, `$majorVersion`, `$minorVersion`, `$patchVersion`, `$cleanVersion`, `$underscoreVersion`, `$dashVersion` template variables
- **FR-007**: System MUST rename `[remote]` to `[checkver]` in all manifests (self-describing)
- **FR-008**: System MUST include `[checkver]` data in the compiled catalog (not stripped)
- **FR-009**: System MUST validate all TOML manifests before compilation (reject invalid)
- **FR-010**: System MUST support tiered hash discovery: URL+regex > JSON endpoint > download+compute
- **FR-011**: System MUST maintain backward compatibility with JSON format during transition
- **FR-012**: System MUST use a single CI job for the full pipeline (check → compile → sign → publish)

### Key Entities

- **CatalogDB**: SQLite database with tables for packages, versions, categories
- **VersionFile**: Per-package per-version JSON with url, sha256, discovered_at, release_notes_url
- **TemplateVariable**: Scoop-style version template ($version, $majorVersion, etc.)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Full pipeline (check all packages → compile → sign → publish) completes in under 30 minutes
- **SC-002**: Catalog.db size is under 5MB for ~95 packages
- **SC-003**: Version checking discovers new versions within 6 hours of release
- **SC-004**: Template variables resolve correctly for 100% of test manifests

## Assumptions

- This spec applies to nightwatch-astro/astro-up-manifests, not the main app repo
- The existing Rust compiler and checker crates are the starting point (spec 001 in manifests repo)
- Minisign private key is stored as a GitHub Actions org secret
- GitHub Releases rolling tag (`catalog/latest`) is updated on each successful pipeline run
- Backward compatibility: JSON manifests.json + versions.json still generated during transition
