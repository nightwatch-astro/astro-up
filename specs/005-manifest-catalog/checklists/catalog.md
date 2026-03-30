# Catalog Requirements Quality Checklist: 005-manifest-catalog

**Purpose**: Validate completeness, clarity, and consistency of manifest catalog requirements before planning
**Created**: 2026-03-30
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Are the Rust structs for reading `packages`, `versions`, `meta` tables specified, or is the schema-to-struct mapping left implicit? [Completeness, Spec §Key Entities]
- [ ] CHK002 - Are requirements for parsing JSON-encoded columns (`tags`, `aliases`, `dependencies`) from SQLite TEXT fields documented? [Completeness, Gap]
- [ ] CHK003 - Is the ETag storage mechanism specified (sidecar file, SQLite table, HTTP cache header file)? [Completeness, Spec §FR-004]
- [ ] CHK004 - Are requirements for the `compiled_at` metadata field defined (display to user, staleness warning, ignore)? [Completeness, Spec §Key Entities]
- [ ] CHK005 - Is the lockfile lifecycle specified — when is it created, when released, what happens on crash (stale lock)? [Completeness, Spec §FR-011]
- [ ] CHK006 - Are requirements for listing all software (unfiltered catalog browse) defined, or only search/filter/resolve? [Completeness, Gap]

## Requirement Clarity

- [ ] CHK007 - Is "configurable source URL" (FR-001) clarified as coming from `CatalogConfig.url` in spec 004, or could it be overridden per-invocation? [Clarity, Spec §FR-001]
- [ ] CHK008 - Is "actionable messages" (FR-010) quantified with specific error scenarios and their message content? [Clarity, Spec §FR-010]
- [ ] CHK009 - Is "major version check" (FR-013) defined precisely — is `schema_version` an integer, semver string, or opaque string? What constitutes "major"? [Clarity, Spec §FR-013]
- [ ] CHK010 - Is the TTL calculation specified — from file modification time, from `compiled_at`, or from last successful fetch time? [Clarity, Spec §FR-003]
- [ ] CHK011 - Is "ranked results" in SC-002 defined — what ranking algorithm does FTS5 use, and are results ordered by relevance or alphabetically? [Clarity, Spec §SC-002]

## Requirement Consistency

- [ ] CHK012 - Are the `Software` struct fields in spec 003 (`software.rs`) consistent with the `packages` table columns the client will read? [Consistency, Spec §Key Entities]
- [ ] CHK013 - Does `PackageId` validation regex in FR-009 match the regex used by the compiler's manifest validation? [Consistency, Spec §FR-009]
- [ ] CHK014 - Are the FTS5 indexed columns in FR-007 (`name, description, tags, aliases, publisher`) consistent with the compiler's actual FTS5 definition? [Consistency, Spec §FR-007]
- [ ] CHK015 - Is the `CatalogSource` entity consistent with `CatalogConfig` from spec 004 (URL, TTL, offline fields)? [Consistency, Spec §Key Entities]

## Acceptance Criteria Quality

- [ ] CHK016 - Is SC-001 ("loads in under 10ms") measurable — does it include schema version check and FTS5 readiness, or just `Connection::open`? [Measurability, Spec §SC-001]
- [ ] CHK017 - Is SC-004 ("avoid unnecessary re-downloads") measurable — what metric proves ETag works (304 response count, bandwidth saved)? [Measurability, Spec §SC-004]
- [ ] CHK018 - Are acceptance scenarios for User Story 1 missing a case for `catalog.offline = true`? [Coverage, Spec §US-1]

## Scenario Coverage

- [ ] CHK019 - Are requirements defined for what happens when the catalog is fetched but signature verification fails AND no previous valid catalog exists? [Coverage, Spec §US-3]
- [ ] CHK020 - Are requirements defined for concurrent fetch attempts (two app instances start simultaneously, both past TTL)? [Coverage, Spec §FR-011]
- [ ] CHK021 - Are requirements for catalog size growth specified — what if the catalog grows from ~100 to 500+ packages? [Coverage, Gap]
- [ ] CHK022 - Are requirements for FTS5 search with special characters defined (dots in "n.i.n.a.", hyphens in package IDs)? [Coverage, Spec §FR-007]

## Edge Case Coverage

- [ ] CHK023 - Is the behavior specified when `meta` table is missing or `schema_version` key is absent (corrupt/incomplete catalog)? [Edge Case, Spec §FR-013]
- [ ] CHK024 - Is the behavior specified when ETag request returns a different content-type or unexpected response (CDN edge case)? [Edge Case, Spec §FR-004]
- [ ] CHK025 - Is the behavior specified when the local catalog file is locked by another process (e.g., antivirus scanning)? [Edge Case, Gap]
- [ ] CHK026 - Is the behavior specified for a catalog where `versions` table has entries for a `package_id` not in `packages`? [Edge Case, Gap]

## Non-Functional Requirements

- [ ] CHK027 - Are retry/backoff requirements defined for failed catalog fetches? [NFR, Gap]
- [ ] CHK028 - Are logging/tracing requirements specified for catalog operations (fetch, verify, load, search)? [NFR, Gap]
- [ ] CHK029 - Is the maximum acceptable catalog file size or download timeout specified? [NFR, Gap]
- [ ] CHK030 - Are requirements for signature verification performance defined (acceptable overhead on top of SC-001)? [NFR, Spec §SC-001]

## Dependencies & Assumptions

- [ ] CHK031 - Is the assumption "FTS5 is available in rusqlite with the `bundled` feature" validated against the project's current rusqlite configuration? [Assumption]
- [ ] CHK032 - Is the dependency on the manifests repo release URL (`catalog/latest`) documented as a deployment prerequisite? [Dependency, Gap]
- [ ] CHK033 - Is the assumption that "package IDs are unique" documented as enforced by the compiler, not the client? [Assumption]
- [ ] CHK034 - Is the one-time manifest rename migration (D5) tracked as a prerequisite or parallel task? [Dependency, Spec §Assumptions]

## Notes

- Check items off as completed: `[x]`
- Add comments or findings inline
- Items are numbered sequentially for easy reference
- Focus: full-spec review at standard depth for reviewer audience
