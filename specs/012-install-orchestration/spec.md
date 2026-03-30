# Feature Specification: Install Orchestration Engine

**Feature Branch**: `012-install-orchestration`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 011 — coordinate check → download → backup → install cycle

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Update a Single Package (Priority: P1)

A user runs `astro-up update nina`. The engine orchestrates: compare catalog version against installed → download installer → backup config → execute installer → verify install. Each step emits events for progress tracking.

**Why this priority**: Single-package update is the most common operation.

**Independent Test**: Update a test package end-to-end. Verify each step executes in order.

**Acceptance Scenarios**:

1. **Given** NINA 3.0 is installed and catalog has 3.1, **When** `update nina` runs, **Then** the engine downloads → backs up → installs 3.1 → verifies
2. **Given** `--dry-run` flag, **When** update runs, **Then** each step is logged but no changes are made
3. **Given** the download fails, **When** the engine detects the failure, **Then** it stops the pipeline (no install attempt)
4. **Given** installed version equals catalog version, **When** update runs, **Then** "already up to date" is reported

---

### User Story 2 - Update All Packages (Priority: P2)

A user runs `astro-up update --all`. The engine updates all packages with available updates, respecting dependency order (prerequisites before dependents).

**Why this priority**: Bulk updates are the main time-saver over manual management.

**Independent Test**: Set up 3 packages with updates (one depends on another), run `update --all`, verify dependency order.

**Acceptance Scenarios**:

1. **Given** 5 packages have updates and 2 have dependencies, **When** `update --all` runs, **Then** dependencies install before dependents
2. **Given** one package fails, **When** detected, **Then** dependent packages are skipped but independent ones continue
3. **Given** `--allow-major` is NOT passed, **When** a major upgrade is available, **Then** it is skipped with a notification

---

### User Story 3 - Version Comparison (Priority: P3)

The engine compares installed versions against catalog versions using format-aware parsing. Semver versions use numeric comparison, date versions use chronological comparison, custom formats use regex-extracted component comparison. The version format is specified per package in the catalog.

**Why this priority**: Correct version comparison drives all update decisions. Wrong comparisons mean missed updates or false positives.

**Independent Test**: Compare versions in each format: semver (3.1.0 < 3.2.0), date (2025.12.01 < 2026.01.01), custom regex.

**Acceptance Scenarios**:

1. **Given** installed 3.1.0 and catalog 3.2.0 (semver), **When** compared, **Then** update is available
2. **Given** installed 2025.12.01 and catalog 2026.03.15 (date format), **When** compared, **Then** update is available
3. **Given** installed 3.2.0 and catalog 3.1.0 (installed is newer), **When** compared, **Then** status is "newer than catalog" (beta/dev build)
4. **Given** no version format specified, **When** comparing, **Then** default to semver with lenient coercion

---

### User Story 4 - Dependency Resolution (Priority: P4)

A package requires ASCOM Platform 7+. The engine checks if the dependency is met, and if not, offers to install it first.

**Why this priority**: Dependency failures are the #1 install issue for astrophotography software.

**Independent Test**: Install a package that depends on ASCOM. Verify the engine installs ASCOM first.

**Acceptance Scenarios**:

1. **Given** NINA requires ASCOM Platform 7+, **When** ASCOM 6.6 is installed, **Then** the engine offers to update ASCOM first
2. **Given** all dependencies are satisfied, **When** installing, **Then** the engine proceeds directly
3. **Given** a circular dependency, **When** detected at plan time, **Then** the engine reports the cycle and aborts

---

### User Story 5 - Update Policy Enforcement (Priority: P5)

The user configures policies: "allow minor updates, require confirmation for major." The engine enforces these during bulk updates.

**Why this priority**: Control over what gets updated automatically.

**Independent Test**: Configure "minor only", verify major updates are skipped.

**Acceptance Scenarios**:

1. **Given** "minor only" default policy, **When** a major update is available, **Then** skipped with notice
2. **Given** a per-package override allowing major for NINA, **When** NINA has a major update, **Then** NINA updates but others respect default
3. **Given** `--allow-major` CLI flag, **When** running, **Then** all major updates proceed for this invocation only

---

### User Story 6 - Operation History (Priority: P6)

Every install, update, and uninstall operation is logged to the local SQLite database with package, versions, status, duration, and timestamp. This history is available for diagnostics and future UI display.

**Why this priority**: Low cost (one INSERT per operation), high future value (debugging, activity log).

**Independent Test**: Run an update, verify an operation record is written with correct fields.

**Acceptance Scenarios**:

1. **Given** a successful update from 3.0 to 3.1, **When** checking history, **Then** a record shows package, from/to versions, "success", and duration
2. **Given** a failed install, **When** checking history, **Then** a record shows "failed" with the error message
3. **Given** a cancelled operation, **When** checking history, **Then** a record shows "cancelled"

### Edge Cases

- Package in use during update: Report the process name, suggest closing it.
- Disk space insufficient: Check before downloading, report required vs available.
- Downgrade request: Reject unless `--allow-downgrade` is explicitly passed.
- Version format mismatch (catalog says semver, detection returns date): Fall back to raw string comparison with a warning.
- Package with no version format in catalog: Default to semver with lenient coercion.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST orchestrate the pipeline: compare versions → download → backup → install → verify
- **FR-002**: System MUST resolve dependencies and install in topological order
- **FR-003**: System MUST enforce update policies (minor-only, major-allowed, manual, none) globally and per-package
- **FR-004**: System MUST support `--dry-run` that reports the plan without making changes
- **FR-005**: System MUST support `--allow-major` to override minor-only policy for a single invocation
- **FR-006**: System MUST emit events for each pipeline step
- **FR-007**: System MUST continue updating independent packages when one fails
- **FR-008**: System MUST skip dependent packages when a dependency fails
- **FR-009**: System MUST verify the installed version after installation (re-detect and compare)
- **FR-010**: System MUST support cancellation at any point
- **FR-011**: System MUST check available disk space before downloading
- **FR-012**: System MUST prevent downgrades unless explicitly requested
- **FR-013**: System MUST compare versions using format-aware parsing: semver, date, custom regex (ported from manifests repo `ParsedVersion`)
- **FR-014**: System MUST detect "newer than catalog" status (installed > latest) and report it distinctly from "up to date"
- **FR-015**: System MUST log every operation to the local SQLite operations table (package, from/to version, status, duration, error, timestamp)
- **FR-016**: System MUST use the `version_format` field from the catalog to select the correct version parser
- **FR-017**: System MUST default to lenient semver when no version format is specified

### Key Entities

- **UpdatePlan**: Ordered list of PackageUpdate items with resolved dependencies
- **PackageUpdate**: Package info, current version, target version, download URL, version format
- **UpdatePolicy**: Default policy + per-package overrides (Minor, Major, Manual, None)
- **PipelineEvent**: Enum of compare/download/backup/install/verify events with progress
- **OperationRecord**: Package ID, operation type, from/to version, status, duration, error, timestamp
- **PackageStatus**: UpToDate, UpdateAvailable, MajorUpgradeAvailable, NewerThanCatalog, NotInstalled

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: End-to-end update for a single package completes in under 3 minutes (excluding download time)
- **SC-002**: Dependency resolution correctly orders all packages
- **SC-003**: Dry-run output matches what a real run would do
- **SC-004**: Version comparison produces correct results for semver, date, and custom formats
- **SC-005**: Operation history is queryable for any package's install/update timeline

## Version Formats

| Format | Manifest Field | Example | Comparison |
|--------|---------------|---------|-----------|
| Semver (default) | `version_format = "semver"` or absent | `3.1.2`, `v2.0.1`, `3.1` | Numeric major.minor.patch |
| Date | `version_format = "date"` | `2026.03.29`, `2025-12-01` | Chronological YYYY.MM.DD |
| Custom | `version_format = "<regex>"` | `3.1 HF2` with `(\d+)\.(\d+) HF(\d+)` | Captured group comparison |
| 4-part | Coerced to semver | `3.1.2.3001` → `3.1.2` | Semver after stripping 4th |

Implementation: Port `ParsedVersion` enum from `nightwatch-astro/astro-up-manifests/crates/shared/src/version.rs` into `astro-up-core`. Replace `lenient_semver` (unmaintained) with the ported lenient parser.

## Assumptions

- Dependency information is declared in manifests, not discovered at runtime
- Sequential installs (one at a time) to avoid Windows installer conflicts
- Backup is optional per package — only runs if `[backup]` is defined
- Operation history is append-only to SQLite — no UI for it in this spec (future CLI `history` command or GUI activity log)
- The `version_format` field is part of the catalog schema (populated from manifest TOML)
- Depends on: spec 005 (catalog for versions/formats), spec 006 (detection), spec 010 (download), spec 011 (install), spec 013 (backup)
