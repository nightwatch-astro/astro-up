# Feature Specification: Manifest Pipeline Modernization

**Feature Branch**: `020-manifest-modernization`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Target Repo**: nightwatch-astro/astro-up-manifests (NOT this repo)
**Input**: Migration plan Spec 019 — modernize manifest pipeline for SQLite catalog distribution

## Note

This spec's implementation lives in **nightwatch-astro/astro-up-manifests**. The spec is tracked here for migration plan completeness. Tasks and issues will be created in the manifests repo.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - SQLite Catalog Compilation (Priority: P1)

The CI pipeline compiles all TOML manifests + per-version JSON files into a single `catalog.db` (SQLite) with FTS5 search indexes. The database is signed with minisign and published to GitHub Releases.

**Why this priority**: The Rust client (spec 005) expects a signed SQLite catalog. This is the critical path for the main app to work.

**Independent Test**: Run the compiler, verify catalog.db contains all packages with correct versions and FTS5 search works.

**Acceptance Scenarios**:

1. **Given** 96 TOML manifests and per-version files, **When** the compiler runs, **Then** catalog.db is produced with packages + versions tables + FTS5 index
2. **Given** catalog.db is compiled, **When** signed, **Then** catalog.db.minisig is produced with the project's minisign key
3. **Given** artifacts are ready, **When** published, **Then** catalog.db + signature are uploaded to a rolling `catalog/latest` GitHub Release
4. **Given** the client fetches catalog.db, **When** querying, **Then** FTS5 search across name, aliases, tags, description works

---

### User Story 2 - Manifest ID Rename (Priority: P2)

All 96 manifest filenames are renamed from `{vendor}-{product}.toml` to short IDs: `nina-app.toml` → `nina.toml`, `phd2-guider.toml` → `phd2.toml`, etc. This aligns with spec 005's decision to merge ID and slug.

**Why this priority**: The client uses the filename (minus .toml) as the package ID. Short IDs are the user-facing identifiers.

**Independent Test**: After rename, verify the compiler produces the same catalog with new IDs. Verify no duplicate IDs.

**Acceptance Scenarios**:

1. **Given** the rename script runs, **When** checking manifests, **Then** all files have short lowercase hyphenated names matching `^[a-z][a-z0-9]*(-[a-z0-9]+)*\.toml$`
2. **Given** renamed manifests, **When** the compiler runs, **Then** catalog.db contains the new IDs
3. **Given** a renamed manifest, **When** checking its `aliases` field, **Then** the old ID is NOT added as an alias (clean break)
4. **Given** per-version files, **When** renamed, **Then** `versions/{old-id}/` directories are renamed to `versions/{new-id}/`

---

### User Story 3 - Per-Version File Storage (Priority: P3)

Each discovered version is stored as an individual JSON file at `versions/{id}/{semver}.json` instead of the monolithic `versions.json`. Git history serves as the audit trail.

**Why this priority**: Per-version files enable cleaner git history and simpler CI (only process changed files).

**Independent Test**: Run the checker for one package, verify a per-version file is created. Run the compiler, verify it reads all per-version files into the catalog.

**Acceptance Scenarios**:

1. **Given** NINA 3.1.2 is discovered, **When** the checker writes, **Then** `versions/nina/3.1.2.json` is created with `{url, sha256, discovered_at}`
2. **Given** the version file already exists, **When** re-discovered, **Then** it is not overwritten (idempotent)
3. **Given** multiple versions exist for a package, **When** the compiler runs, **Then** only the latest version per package is included in catalog.db

---

### User Story 4 - Remove Committed Build Artifacts (Priority: P4)

`manifests.json`, `versions.json`, `stats.json`, and their signatures are removed from the git repository. They become CI-only build artifacts published to GitHub Releases. The repo contains only source files (TOMLs + per-version JSONs).

**Why this priority**: Build artifacts in git create merge conflicts, bloat the repo, and confuse the source-of-truth.

**Independent Test**: After cleanup, verify `git status` shows no JSON build artifacts. Verify CI still produces and publishes them.

**Acceptance Scenarios**:

1. **Given** the cleanup runs, **When** checking git, **Then** `manifests.json`, `versions.json`, `stats.json`, and `.minisig` files are deleted from the repo
2. **Given** `.gitignore` is updated, **When** the compiler outputs these files, **Then** they are not committed
3. **Given** the docs site needs `stats.json`, **When** CI runs, **Then** `stats.json` is published to GitHub Releases alongside catalog.db

### Edge Cases

- Manifest ID collision after rename: CI validates uniqueness before merging the rename PR.
- Per-version file for a package that was removed from manifests: Orphan version files are ignored by the compiler (no matching manifest).
- Docs site fetches stats.json: Update the docs site to fetch from GitHub Releases instead of raw.githubusercontent.com/main.
- Backward compat for old Go client: Not needed — Go client is archived. Clean break.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Compiler MUST produce a SQLite catalog.db with packages table, versions table, and FTS5 index
- **FR-002**: Compiler MUST read per-version files from `versions/{id}/{semver}.json` as version source
- **FR-003**: Compiler MUST include only the latest version per package in catalog.db
- **FR-004**: Compiler MUST include `version_format` field per package for client-side version comparison (spec 012)
- **FR-005**: CI MUST sign catalog.db with minisign and publish to GitHub Releases (`catalog/latest` rolling tag)
- **FR-006**: CI MUST publish stats.json to the same GitHub Release (for docs site)
- **FR-007**: All 96 manifests MUST be renamed to short IDs matching `^[a-z][a-z0-9]*(-[a-z0-9]+)*\.toml$`
- **FR-008**: Per-version directories MUST be renamed to match new manifest IDs
- **FR-009**: `manifests.json`, `versions.json`, `stats.json`, and `.minisig` files MUST be removed from git and added to `.gitignore`
- **FR-010**: Checker MUST write discovered versions to `versions/{id}/{semver}.json` (not monolithic versions.json)
- **FR-011**: Checker MUST be idempotent — don't overwrite existing per-version files
- **FR-012**: CI MUST run as a single job: check → compile → sign → publish

### Catalog Schema

```sql
CREATE TABLE packages (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    type TEXT NOT NULL,
    publisher TEXT,
    description TEXT,
    homepage TEXT,
    license TEXT,
    aliases TEXT,           -- JSON array
    tags TEXT,              -- JSON array
    manifest TEXT NOT NULL, -- Full manifest as JSON (for detection config, install config, etc.)
    version_format TEXT     -- 'semver' | 'date' | regex pattern
);

CREATE TABLE versions (
    package_id TEXT NOT NULL REFERENCES packages(id),
    version TEXT NOT NULL,
    url TEXT,
    sha256 TEXT,
    discovered_at TEXT NOT NULL,
    PRIMARY KEY (package_id, version)
);

CREATE VIRTUAL TABLE packages_fts USING fts5(
    id, name, aliases, tags, description,
    content='packages', content_rowid='rowid'
);
```

### Key Entities

- **CatalogDB**: SQLite database with packages + versions + FTS5 index
- **PerVersionFile**: `versions/{id}/{semver}.json` with url, sha256, discovered_at
- **ManifestRename**: Mapping from old IDs to new short IDs

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: catalog.db size under 5MB for 96 packages
- **SC-002**: FTS5 search returns results in under 10ms
- **SC-003**: Full pipeline (check + compile + sign + publish) completes in under 30 minutes
- **SC-004**: All 96 manifests renamed with zero ID collisions

## Assumptions

- Implementation is in nightwatch-astro/astro-up-manifests, not this repo
- The existing Rust compiler and checker crates are the starting point
- No backward compatibility with Go client (archived)
- No backward compatibility with old manifest IDs (clean break)
- The docs site (astro-up.github.io) must be updated to fetch stats.json from GitHub Releases
- Depends on: spec 005 (catalog format consumed by client), spec 012 (version_format field)
