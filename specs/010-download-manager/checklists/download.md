# Download Manager Requirements Quality Checklist

**Purpose**: Validate requirement completeness, clarity, and consistency for the download system
**Created**: 2026-03-30
**Feature**: [spec.md](../spec.md)
**Focus**: Network reliability, data integrity, resource management, recovery flows

## Requirement Completeness

- [x] CHK001 - Are requirements defined for what happens when a download URL returns a redirect chain longer than 10 hops? [Completeness, Spec §FR-010] — Error reported
- [x] CHK002 - Does the spec define the download directory structure — flat or nested by package/version? [Gap] — Flat: `{download_dir}/{filename}` (FR-014)
- [x] CHK003 - Are requirements defined for concurrent download requests from different callers? [Gap] — Sequential enforced by download manager, concurrent callers rejected (FR-012, Assumptions)
- [x] CHK004 - Is the `.part` file naming convention specified? [Completeness, Spec §FR-007] — `{final_filename}.part`
- [x] CHK005 - Are requirements defined for what "atomic rename" means on Windows vs Unix? [Completeness, Spec §FR-007] — Retry up to 3 times with 1s delay if locked
- [x] CHK006 - Does the spec define when the User-Agent version is constructed? [Clarity, Spec §FR-018] — Build time via `env!("CARGO_PKG_VERSION")`

## Requirement Clarity

- [x] CHK007 - Is "configurable request timeouts" specified with separate connect vs read timeout? [Clarity, Spec §FR-009] — Connect 10s, read 30s
- [x] CHK008 - Is throttle measurement window defined? [Clarity, Spec §SC-005] — 5-second rolling average
- [x] CHK009 - Is disk space check specified for unknown file size? [Clarity, Spec §FR-017] — Skip check when Content-Length unknown
- [x] CHK010 - Are progress events specified for very slow downloads? [Clarity, Spec §SC-004] — Emit at least once per second; with no data, emit with current bytes_downloaded unchanged

## Recovery & Resume Coverage

- [x] CHK011 - Are requirements defined for corrupt `.part` files? [Gap, Recovery] — Validate size against offset, delete if inconsistent
- [x] CHK012 - Is behavior specified when resumed download hash fails? [Gap, Spec §FR-002] — Retry once from scratch, then error
- [x] CHK013 - Are requirements defined for server file change between attempts? [Coverage, Spec §FR-006] — Last-Modified freshness check catches this, restart from scratch
- [x] CHK014 - Is ETag vs resume usage clarified? [Clarity, Spec §FR-008] — ETag for fresh downloads, Range for resume

## Edge Case Coverage

- [x] CHK015 - Are requirements defined for no Content-Length? [Gap, Edge Case] — total_bytes=0 (indeterminate), skip disk check, hash still verified
- [x] CHK016 - Is rename failure behavior specified? [Gap, Spec §FR-007] — Retry 3x with 1s delay
- [x] CHK017 - Are URL encoding issues addressed? [Gap, Edge Case] — URLs used as-is from catalog (already encoded)
- [x] CHK018 - Does spec define behavior when download directory doesn't exist? [Gap] — Auto-create (FR-019)

## Non-Functional Requirements

- [x] CHK019 - Is maximum concurrent connection count specified? [Gap, Non-Functional] — One active download enforced by download manager
- [x] CHK020 - Are memory usage requirements defined? [Gap, Spec §FR-001] — 64KB streaming chunk size
- [x] CHK021 - Is broadcast channel capacity specified? [Gap, Non-Functional] — Capacity 64 (FR-003)

## Dependencies & Assumptions

- [x] CHK022 - Is sequential download enforced by download manager or orchestration? [Assumption] — Download manager enforces, concurrent callers rejected
- [x] CHK023 - Are config key names consistent with spec 004? [Consistency, Assumption] — Yes: `network.*` for network, `paths.*` for directories
- [x] CHK024 - Is purge ownership clarified? [Clarity, Spec §FR-015] — Download manager owns `purge()` method, background service (spec 016) calls it

## Notes

- All 24 items resolved in spec update
- FR-019 added (auto-create download directory)
- Default values table expanded with connect timeout, chunk size, channel capacity
- Recovery flows fully specified: corrupt .part, hash mismatch after resume, file change between attempts
