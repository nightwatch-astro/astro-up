# Feature Specification: Manifest Parsing and Catalog

**Feature Branch**: `005-manifest-catalog`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 004 — parse TOML manifests, compile to catalog, load from signed JSON/SQLite

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Load Software Catalog (Priority: P1)

A user launches astro-up and sees the full list of available astrophotography software. The application fetches the signed catalog from the official source, verifies the signature, caches it locally, and presents the software list. On subsequent launches, the cached catalog is used unless the TTL has expired.

**Why this priority**: The catalog is the foundation — every other feature (check, update, install) depends on having a loaded catalog.

**Independent Test**: Start the application, verify it fetches and displays the catalog with correct software names, categories, and versions.

**Acceptance Scenarios**:

1. **Given** a fresh install with no cache, **When** the application starts, **Then** it fetches the catalog from the configured source URL and displays all available software
2. **Given** a cached catalog within its TTL, **When** the application starts, **Then** it uses the cache without making a network request
3. **Given** a cached catalog past its TTL, **When** the application starts, **Then** it sends a conditional request (ETag) and uses 304 Not Modified or fetches the new catalog
4. **Given** no network connectivity and a cached catalog, **When** the application starts, **Then** it uses the stale cache with a warning

---

### User Story 2 - Search and Filter Software (Priority: P2)

A user searches for software by name, category, or tag. They type "plate" and see PlateSolve, ASTAP, and All-Sky Plate Solver. They filter by category "guiding" and see PHD2 and MetaGuide.

**Why this priority**: Users need to find specific software quickly. Search and filter are the primary catalog interaction patterns.

**Independent Test**: Load catalog, search for "NINA", verify it appears. Filter by "capture" category, verify only capture software is shown.

**Acceptance Scenarios**:

1. **Given** a loaded catalog, **When** the user searches for "nina", **Then** N.I.N.A. appears in the results (case-insensitive, matches name, aliases, tags)
2. **Given** a loaded catalog, **When** the user filters by category "guiding", **Then** only guiding software is shown
3. **Given** a loaded catalog, **When** the user searches for a non-existent term, **Then** an empty result is returned with no error

---

### User Story 3 - Signature Verification (Priority: P3)

The application verifies that the catalog was signed by the trusted publisher before using it. If the signature is invalid or missing, the application refuses to use the catalog and reports the issue.

**Why this priority**: Supply-chain security — prevents tampered catalogs from triggering malicious installs.

**Independent Test**: Load a catalog with a valid signature — succeeds. Modify one byte — verification fails with a clear error.

**Acceptance Scenarios**:

1. **Given** a catalog with a valid minisign signature, **When** the application verifies it, **Then** the catalog is accepted and loaded
2. **Given** a catalog with an invalid signature, **When** the application verifies it, **Then** it rejects the catalog with a signature verification error
3. **Given** a catalog with no signature file, **When** the application attempts to load it, **Then** it reports a missing signature error

---

### User Story 4 - Resolve Software by ID or Slug (Priority: P4)

A CLI user runs `astro-up check nina-app` or `astro-up check nina` to look up software by its exact ID or human-friendly slug. The resolver matches IDs exactly and slugs with fuzzy tolerance.

**Why this priority**: Direct lookup is needed for CLI commands and internal references between manifests.

**Independent Test**: Resolve "nina-app" by ID — exact match. Resolve "nina" by slug — matches.

**Acceptance Scenarios**:

1. **Given** a loaded catalog, **When** resolving by exact ID "nina-app", **Then** the N.I.N.A. software entry is returned
2. **Given** a loaded catalog, **When** resolving by slug "nina", **Then** the N.I.N.A. software entry is returned
3. **Given** a loaded catalog, **When** resolving an unknown ID, **Then** a not-found error is returned

### Edge Cases

- What happens when the catalog source URL is unreachable and there's no cache? Report network error and exit gracefully.
- What happens when the cached catalog is corrupted? Signature check fails, discard cache, re-fetch.
- What happens when two software entries have the same slug? Compiler rejects duplicate slugs at build time.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST fetch the catalog from a configurable source URL (default: GitHub Releases asset)
- **FR-002**: System MUST verify catalog integrity using minisign signature verification before loading
- **FR-003**: System MUST cache the catalog locally with a configurable TTL (default: 24 hours)
- **FR-004**: System MUST use conditional HTTP requests (ETag/If-None-Match) to avoid re-downloading unchanged catalogs
- **FR-005**: System MUST fall back to the stale cache when the network is unavailable
- **FR-006**: System MUST support resolving software by exact ID, slug, or fuzzy name match
- **FR-007**: System MUST support filtering software by category, type, and tags
- **FR-008**: System MUST parse TOML manifest files into the Software type defined in spec 003
- **FR-009**: System MUST expand path tokens in manifest values at parse time
- **FR-010**: System MUST report parse errors with manifest filename and specific field/line
- **FR-011**: System MUST support an offline mode that uses only cached data
- **FR-012**: System MUST embed the trusted minisign public key at compile time

### Key Entities

- **Catalog**: Full collection of software entries with lookup, search, and filter operations
- **CatalogSource**: Configuration for fetch location (URL, public key, TTL)
- **CacheEntry**: Cached catalog with metadata (fetched_at, etag, expires_at)
- **SearchResult**: Ranked list of matching software entries with relevance scores

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Catalog loads from cache in under 50ms
- **SC-002**: Fuzzy search returns relevant results for partial name matches
- **SC-003**: Invalid signatures are rejected 100% of the time
- **SC-004**: Application remains functional in offline mode using cached data

## Clarifications

- **Catalog format detection**: The client determines format by file extension — `.db` for SQLite, `.json` for JSON. The configured URL dictates the format. No runtime sniffing.
- **Concurrent cache access**: File-level locking on the cache file. If CLI and GUI run simultaneously, second reader waits or uses stale cache. No shared-memory cache between processes.
- **meta.json deferred**: The migration plan mentions a lightweight `meta.json` for change detection. This is deferred — ETag on the catalog URL achieves the same goal with less complexity.
- **Catalog includes version entries**: The SQLite catalog bundles both manifest metadata AND latest version info. The client does NOT need to fetch `versions.json` separately — that's the manifest repo's concern.

## Assumptions

- Catalog artifact is a signed SQLite database on GitHub Releases (JSON fallback during transition only)
- Minisign public key is baked into the binary — no key rotation in this spec
- Fuzzy search uses case-insensitive substring matching; FTS5 available when using SQLite format
- Depends on: spec 003 (types), spec 004 (configuration for catalog URL/TTL)
