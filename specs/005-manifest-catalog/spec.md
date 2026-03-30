# Feature Specification: Manifest Parsing and Catalog

**Feature Branch**: `005-manifest-catalog`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 004 — parse TOML manifests, compile to SQLite catalog, fetch and verify

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Load Software Catalog (Priority: P1)

A user launches astro-up and sees the full list of available astrophotography software. The application fetches the signed SQLite catalog from GitHub Releases, verifies the minisign signature, and stores it locally. On subsequent launches, the local catalog is used unless the TTL has expired, in which case a conditional ETag request checks for updates.

**Why this priority**: The catalog is the foundation — every other feature (check, update, install) depends on having a loaded catalog.

**Independent Test**: Start the application, verify it fetches and displays the catalog with correct software names, categories, and versions.

**Acceptance Scenarios**:

1. **Given** a fresh install with no local catalog, **When** the application starts, **Then** it fetches catalog.db from the configured source URL, verifies the signature, and stores it locally
2. **Given** a local catalog within its TTL, **When** the application starts, **Then** it uses the local file without making a network request
3. **Given** a local catalog past its TTL, **When** the application starts, **Then** it sends a conditional request (ETag) and either keeps the current file (304) or downloads the new one
4. **Given** no network connectivity and a local catalog exists, **When** the application starts, **Then** it uses the local catalog (it's still valid data, just potentially stale)
5. **Given** no network and no local catalog, **When** the application starts, **Then** it reports "no catalog available — check your network" and exits

---

### User Story 2 - Search and Filter Software (Priority: P2)

A user searches for software by name, alias, or tag. They type "plate" and see PlateSolve, ASTAP, and All-Sky Plate Solver. They filter by category "guiding" and see PHD2 and MetaGuide. Search uses SQLite FTS5 for ranked full-text matching.

**Why this priority**: Users need to find specific software quickly. Search and filter are the primary catalog interaction patterns.

**Independent Test**: Load catalog, search for "NINA", verify it appears. Search for "n.i.n.a.", verify alias matching works. Filter by "capture" category, verify only capture software is shown.

**Acceptance Scenarios**:

1. **Given** a loaded catalog, **When** the user searches for "nina", **Then** N.I.N.A. appears in the results (matches ID, name, aliases, and tags via FTS5)
2. **Given** a loaded catalog, **When** the user searches for "n.i.n.a.", **Then** N.I.N.A. appears (alias match)
3. **Given** a loaded catalog, **When** the user filters by category "guiding", **Then** only guiding software is shown
4. **Given** a loaded catalog, **When** the user searches for a non-existent term, **Then** an empty result is returned with no error

---

### User Story 3 - Signature Verification (Priority: P3)

The application verifies that the catalog was signed by the trusted publisher before using it. If the signature is invalid or missing, the application refuses to use the catalog and reports the issue. The minisign public key is embedded in the binary at compile time.

**Why this priority**: Supply-chain security — prevents tampered catalogs from triggering malicious installs.

**Independent Test**: Load a catalog with a valid signature — succeeds. Modify one byte — verification fails with a clear error.

**Acceptance Scenarios**:

1. **Given** a catalog with a valid minisign signature, **When** the application verifies it, **Then** the catalog is accepted and stored
2. **Given** a catalog with an invalid signature, **When** the application verifies it, **Then** it rejects the catalog with a signature verification error and keeps the previous valid catalog if one exists
3. **Given** a catalog with no signature file, **When** the application attempts to load it, **Then** it reports a missing signature error

---

### User Story 4 - Resolve Software by ID (Priority: P4)

A CLI user runs `astro-up check nina` to look up software by its ID. The ID is the canonical short identifier — the same as the manifest filename without the `.toml` extension. Aliases provide additional search terms but are not used for direct resolution.

**Why this priority**: Direct lookup is needed for CLI commands and internal references between manifests.

**Independent Test**: Resolve "nina" by ID — exact match. Resolve "nighttime-imaging" — no match (aliases are for search, not resolution).

**Acceptance Scenarios**:

1. **Given** a loaded catalog, **When** resolving by exact ID "nina", **Then** the N.I.N.A. software entry is returned
2. **Given** a loaded catalog, **When** resolving an unknown ID "nonexistent", **Then** a not-found error is returned
3. **Given** a loaded catalog, **When** searching (not resolving) for "nighttime-imaging", **Then** N.I.N.A. is found via its alias

### Edge Cases

- Catalog source URL unreachable, no local catalog: Error with "no catalog available — check your network."
- Local catalog file corrupted (SQLite integrity check fails): Delete it, re-fetch. If re-fetch also fails, error.
- Duplicate IDs: Rejected at compile time by the manifest repo CI. The client assumes IDs are unique.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST fetch the catalog as a signed SQLite database from a configurable source URL
- **FR-002**: System MUST verify catalog integrity using minisign signature verification with a compile-time embedded public key
- **FR-003**: System MUST store the catalog locally with a configurable TTL (default: 24 hours)
- **FR-004**: System MUST use conditional HTTP requests (ETag/If-None-Match) to avoid re-downloading unchanged catalogs
- **FR-005**: System MUST use the local catalog when network is unavailable (no explicit offline mode — the local file IS the offline catalog)
- **FR-006**: System MUST resolve software by exact ID (case-sensitive, canonical identifier)
- **FR-007**: System MUST search software via FTS5 full-text search across name, aliases, tags, and description
- **FR-008**: System MUST support filtering software by category and type
- **FR-009**: System MUST validate package IDs against the regex `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` (lowercase, hyphen-separated, 2-50 chars)
- **FR-010**: System MUST report catalog fetch/verify errors with actionable messages
- **FR-011**: System MUST use an apt-style PID lockfile (`{data_dir}/astro-up/astro-up.lock`) for write operations to prevent concurrent modification
- **FR-012**: System MUST bundle manifest metadata and version info in the same catalog (one fetch, one file)

### Key Entities

- **Catalog**: SQLite database containing packages table (manifest metadata) and versions table (latest version per package). Provides resolve, search, and filter operations via SQL queries.
- **PackageId**: Validated string matching `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`. The canonical identifier. Also the manifest filename without `.toml`.
- **CatalogSource**: URL + TTL from config (spec 004). ETag stored alongside the local catalog file.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Catalog loads from local SQLite in under 10ms
- **SC-002**: FTS5 search returns ranked results in under 50ms for any query
- **SC-003**: Invalid signatures are rejected 100% of the time
- **SC-004**: ETag conditional requests avoid unnecessary re-downloads

## Assumptions

- Catalog is SQLite only — no JSON format, no backward compatibility with Go client
- The catalog SQLite file on disk IS the cache — no separate cache layer
- Minisign public key is embedded at compile time, not configurable
- FTS5 is available in rusqlite with the `bundled` feature (statically linked SQLite)
- Package IDs are the short, human-friendly name (e.g., `nina`, `phd2`, `ascom-platform`) — no `{vendor}-{product}` convention
- Aliases are search/display terms only, not resolution identifiers
- One-time manifest rename in astro-up-manifests repo: `nina-app.toml` → `nina.toml` etc.
- Depends on: spec 003 (types), spec 004 (configuration for catalog URL/TTL)
