# Feature Specification: Manifest Pipeline Modernization

**Feature Branch**: `002-manifest-pipeline`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 019 — modernize the manifest repository with Scoop/winget-inspired patterns

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Self-Describing Manifest Format (Priority: P1)

A manifest author writes a TOML file for a new astrophotography application. The manifest contains both the software metadata (name, category, detection, install config) AND the version checking configuration (`[checkver]` section) in a single file. The `[checkver]` section uses Scoop-style `$version` template variables for URL construction and tiered hash discovery. The compiler reads this manifest and includes the checkver data in the compiled output — nothing is stripped.

**Why this priority**: The manifest format is the data contract that all other specs depend on. Every downstream consumer (catalog, providers, download, install) reads this format.

**Independent Test**: Write a manifest TOML for NINA with `[checkver]` section using `$version` variables. Compile it. Verify the compiled output retains the checkver configuration.

**Acceptance Scenarios**:

1. **Given** a manifest TOML with `[checkver]` section, **When** the compiler processes it, **Then** the checkver data is preserved in the compiled SQLite output (not stripped)
2. **Given** a manifest using `$version` in a URL template, **When** the version is `3.1.2`, **Then** `$version` resolves to `3.1.2`, `$majorVersion` to `3`, `$cleanVersion` to `312`, `$underscoreVersion` to `3_1_2`, `$dashVersion` to `3-1-2`
3. **Given** a manifest with `hash.url` and `hash.regex`, **When** the checker runs, **Then** it fetches the hash URL and extracts the SHA256 using the regex pattern

---

### User Story 2 - TOML to SQLite Compilation (Priority: P2)

A CI pipeline compiles 95+ TOML manifests into a single SQLite database (`catalog.db`). The SQLite database contains all manifest data plus discovered version information, enabling the client app to query software by ID, category, name, or fuzzy search without JSON parsing. The compiled artifact is signed with minisign and published as a GitHub Release asset.

**Why this priority**: The SQLite compilation replaces the current `manifests.json` + `versions.json` dual-file approach, simplifying the client's data access pattern.

**Independent Test**: Run the compiler against the manifests directory. Verify the output `catalog.db` contains all 95 manifests queryable by ID and category.

**Acceptance Scenarios**:

1. **Given** a directory of TOML manifests, **When** the compiler runs, **Then** it produces a `catalog.db` SQLite file containing all manifest data
2. **Given** the compiled `catalog.db`, **When** querying by category "capture", **Then** it returns all capture software entries with their full metadata
3. **Given** a new version discovered for a package, **When** the compiler runs, **Then** the version data from `versions/{id}/{semver}.json` is imported into the SQLite database
4. **Given** the compiled `catalog.db`, **When** signing with minisign, **Then** it produces `catalog.db.minisig` verifiable with the public key

---

### User Story 3 - Per-Version File Storage (Priority: P3)

When the CI version checker discovers a new version of a package, it writes a JSON file at `versions/{package-id}/{semver}.json` containing the download URL, SHA256 hash, discovery timestamp, and release notes URL. Git history provides the full audit trail. The compiler aggregates all per-version files into the SQLite database.

**Why this priority**: Per-version files replace the flat `versions.json`, enabling granular version history with git as the audit trail and supporting rollback to any previously-discovered version.

**Independent Test**: Create a per-version file for NINA 3.1.2. Run the compiler. Verify the version appears in the SQLite database with correct URL and hash.

**Acceptance Scenarios**:

1. **Given** the checker discovers NINA version 3.1.2, **When** it writes the version file, **Then** `versions/nina-app/3.1.2.json` contains `{ "url": "...", "sha256": "...", "discovered_at": "...", "release_notes_url": "..." }`
2. **Given** multiple version files exist for a package, **When** the compiler runs, **Then** the SQLite database contains all versions ordered by semver
3. **Given** a version file was written 6 months ago, **When** querying the git log for that file, **Then** the discovery date and commit context are available as audit trail

---

### User Story 4 - Simplified CI Pipeline (Priority: P4)

The version checking and compilation pipeline runs as a single GitHub Actions job on a 6-hour cron schedule. It iterates all manifests, runs checkver for each, writes new version files, compiles to SQLite, signs the artifact, and publishes it as a GitHub Release asset on a rolling `catalog/latest` tag.

**Why this priority**: Replacing the current 3-job matrix (resolve → check → merge) with a single job reduces CI complexity and cost.

**Independent Test**: Trigger the CI workflow manually. Verify it checks versions, compiles, signs, and publishes the `catalog.db` + `catalog.db.minisig` assets.

**Acceptance Scenarios**:

1. **Given** the CI runs on schedule, **When** a new version is discovered, **Then** the version file is written, SQLite is recompiled, signed, and uploaded to the `catalog/latest` release
2. **Given** a vendor website is unreachable, **When** the checker runs, **Then** it logs the failure and continues to the next manifest (no pipeline abort)
3. **Given** no new versions are discovered, **When** the pipeline completes, **Then** no new commit is created and the release assets remain unchanged
4. **Given** the `catalog/latest` release exists, **When** the pipeline uploads new assets, **Then** the old assets are replaced via `gh release upload --clobber`

---

### User Story 5 - Backward Compatible Transition (Priority: P5)

During the transition period, the pipeline produces both the new SQLite artifact AND the legacy `manifests.json` + `versions.json` files. The old Go client continues to work while the new Rust client consumes the SQLite database. Once the Rust client reaches feature parity, the legacy JSON output is removed.

**Why this priority**: Ensures zero downtime during migration — both old and new clients work simultaneously.

**Independent Test**: Run the pipeline. Verify both `catalog.db` and `manifests.json` are produced with consistent data.

**Acceptance Scenarios**:

1. **Given** the pipeline runs, **When** compilation completes, **Then** both `catalog.db` and `manifests.json` are produced
2. **Given** the old Go client fetches `manifests.json`, **When** comparing with the new format, **Then** the software entries are identical (minus checkver data that was previously stripped)
3. **Given** the Rust client fetches `catalog.db`, **When** querying for NINA, **Then** the result matches the Go client's view plus additional checkver metadata

---

### Edge Cases

- What happens when a manifest TOML has invalid syntax? The compiler MUST report the error with file path and line number, skip the invalid manifest, and continue processing.
- What happens when a `$version` variable is used but the version string has no minor component (e.g., "3")? The variable MUST resolve to empty string for missing components (`$minorVersion` = "", `$patchVersion` = "").
- What happens when the SHA256 hash from `hash.url` doesn't match the downloaded file? The version file MUST NOT be written. The mismatch MUST be logged as an error and an issue auto-created.
- What happens when the GitHub Release `catalog/latest` doesn't exist on first run? The pipeline MUST create it before uploading assets.
- What happens when the manifest repo has 200+ manifests? The pipeline MUST complete within 15 minutes on a standard GitHub Actions runner.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Each TOML manifest MUST include a `manifest_version` field for schema versioning (starting at `1`)
- **FR-002**: The `[remote]` section MUST be renamed to `[checkver]` across all manifests. The checkver section MUST remain in the compiled output (not stripped)
- **FR-003**: URL templates in `[checkver]` and `[checkver.autoupdate]` MUST support Scoop-style variable substitution: `$version`, `$majorVersion`, `$minorVersion`, `$patchVersion`, `$cleanVersion`, `$underscoreVersion`, `$dashVersion`, `$preReleaseVersion`, `$buildVersion`
- **FR-004**: Hash discovery MUST support a tiered approach: (1) `hash.url` + `hash.regex` — fetch URL, extract hash via regex; (2) `hash.jsonpath` — fetch JSON, extract via path; (3) download the file and compute SHA256 (fallback for new versions only)
- **FR-005**: Default installer switches MUST be defined per installer type (InnoSetup: `/VERYSILENT /NORESTART /SUPPRESSMSGBOXES`, MSI: `/qn /norestart`, etc.) to reduce manifest verbosity. Manifests MAY override defaults.
- **FR-006**: The compiler MUST produce a SQLite database (`catalog.db`) with a pragmatic normalized schema (8 tables): `packages` (metadata columns + JSON for tags, aliases, dependencies), `detection` (method + method-specific columns, fallback fields flattened), `install` (method, scope, elevation + JSON for switches, exit_codes, success_codes), `checkver` (provider, owner, repo, url, regex + JSON for autoupdate, hash), `hardware` (device_class, inf_provider + JSON for vid_pid), `backup` (JSON for config_paths), `versions` (package_id + version as PK, url, sha256, discovered_at, release_notes_url), `meta` (key-value for schema version and compilation timestamp). Indexes on `packages.category`, `packages.type`, `packages.slug`. FTS5 virtual table on `name`, `description`, `tags`, `publisher` for fuzzy search.
- **FR-007**: The compiler MUST produce a `catalog.db.minisig` signature file using the CI's minisign private key
- **FR-008**: Discovered versions MUST be stored as individual JSON files at `versions/{package-id}/{semver}.json` with fields: `url`, `sha256`, `discovered_at`, `release_notes_url`
- **FR-009**: The CI pipeline MUST run as a single job: iterate manifests → checkver → write version files → compile SQLite → sign → publish to GitHub Releases (`catalog/latest` tag, `--clobber`)
- **FR-010**: The CI pipeline MUST run on a 6-hour cron schedule and on manual dispatch with optional vendor/category filter
- **FR-011**: The pipeline MUST auto-create GitHub issues when a vendor check fails persistently and auto-close when resolved
- **FR-012**: During the transition period, the compiler MUST also produce `manifests.json` (legacy format, checkver stripped) for backward compatibility with the Go client
- **FR-013**: The manifest TOML format MUST include a `[hardware]` section for driver packages: `vid_pid` (USB VID:PID patterns), `device_class`, `inf_provider`
- **FR-014**: The compiler and checker MUST be implemented as Rust binaries in a separate Cargo workspace in the manifest repository (replacing the current Go modules)
- **FR-017**: Manifests MUST be validated at compilation time via typed Rust struct deserialization (`serde` + `validator`/`garde`). Invalid manifests MUST be skipped with a clear error (file path + field name), not abort the entire compilation. CI MUST validate manifests on every PR via the compiler in dry-run mode (`--validate`)
- **FR-015**: The Rust checker MUST support all existing check methods: `github`, `gitlab`, `direct_url`, `http_head`, `go_scrape` (renamed to `html_scrape`), `rod_scrape` (renamed to `browser_scrape`), `pe_download`, `manual`
- **FR-016**: The checker MUST run checks in parallel (configurable concurrency, default 10) using authenticated GitHub App token requests. Failed checks MUST retry with exponential backoff (3 attempts, 1s/2s/4s). Rate limit responses (HTTP 429, GitHub `retry-after`) MUST be respected by pausing that provider until the reset window

### Key Entities

- **Manifest**: A TOML file defining a software package — metadata, detection, install, checkver, hardware, backup config
- **Version Entry**: A JSON file recording a discovered version — URL, SHA256, timestamp, release notes link
- **Catalog Database**: The compiled SQLite artifact containing all manifests + versions, served via GitHub Releases
- **Check Method**: A strategy for discovering the latest version of a package (GitHub API, HTML scraping, PE download, etc.)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 95 existing manifests are migrated to the new format (`[remote]` → `[checkver]`, `manifest_version` field added) without data loss
- **SC-002**: The compiled `catalog.db` is smaller than or equal to the combined size of `manifests.json` + `versions.json` (~110 KB)
- **SC-003**: The single-job CI pipeline completes in under 15 minutes for all 95 manifests
- **SC-004**: The client can fetch `catalog.db` via ETag conditional request and skip download when unchanged
- **SC-005**: The old Go client continues to function during the transition period using `manifests.json`
- **SC-006**: Version discovery accuracy matches or exceeds the current Go checker (same versions found for the same manifests)

## Clarifications

### Session 2026-03-29

- Q: SQLite catalog schema — normalized, hybrid, or denormalized? → A: Pragmatic normalized (Option D). 8 tables: `packages`, `detection`, `install`, `checkver`, `hardware`, `backup`, `versions`, `meta`. Top-level config fields are proper columns; only arrays/maps (switches, exit_codes, tags, aliases, vid_pid, config_paths, autoupdate, hash) stay as JSON. FTS5 for fuzzy search. Revised from initial hybrid (Option B) after deeper analysis — JSON-only columns underuse SQLite.
- Q: GitHub API rate limiting strategy for CI checker? → A: Parallel checks (configurable concurrency, default 10) with authenticated GitHub App token. Exponential backoff retries (3 attempts, 1s/2s/4s). Respect HTTP 429 and retry-after headers.
- Q: Where do the Rust checker/compiler binaries live? → A: Separate Cargo workspace in the manifest repo. Independent CI, own dependencies, no coupling to main app workspace. Chromium dependency stays isolated.
- Q: Browser scraping strategy for Rust checker? → A: `chromiumoxide` crate — modern async CDP client, tokio-native. Replaces Go Rod. CI caches Chromium binary.
- Q: Manifest validation strategy? → A: Typed Rust structs with serde + validator/garde. Validated at compile time (compiler binary run, not Rust compilation). Invalid manifests skipped with clear errors, never abort entire compilation. CI validates on PR via `--validate` flag.

## Assumptions

- The manifest repository will be migrated to `nightwatch-astro/astro-up-manifests` (new org)
- The minisign private key is stored as a GitHub secret in the manifest repo
- The GitHub App token (`NIGHTWATCH_APP_ID` + `NIGHTWATCH_APP_PRIVATE_KEY`) is available for CI operations
- Browser-based scraping (`browser_scrape`) continues to use Chromium/Rod pattern (heavy dependency stays in the manifest repo CI, not the client)
- The `catalog.db` schema is versioned separately from the manifest schema — breaking changes to the DB schema require a version bump
- The client app (Spec 004 — Catalog) handles fetching and querying the SQLite database
