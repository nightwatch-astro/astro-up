# Feature Specification: Install Orchestration Engine

**Feature Branch**: `012-install-orchestration`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 011 — coordinate check → download → backup → install cycle

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Update a Single Package (Priority: P1)

A user runs `astro-up update nina-app`. The engine orchestrates: check latest version → download installer → backup config → execute installer → verify install. Each step emits events for progress tracking.

**Why this priority**: Single-package update is the most common operation.

**Independent Test**: Update a test package end-to-end. Verify each step executes in order and the package is updated.

**Acceptance Scenarios**:

1. **Given** NINA 3.0 is installed and 3.1 is available, **When** `update nina-app` runs, **Then** the engine checks → downloads → backs up → installs 3.1
2. **Given** the user runs with `--dry-run`, **When** update runs, **Then** each step is logged but no changes are made
3. **Given** the download fails, **When** the engine detects the failure, **Then** it stops the pipeline and reports the error (no install attempt)

---

### User Story 2 - Update All Packages (Priority: P2)

A user runs `astro-up update --all`. The engine updates all packages with available updates, respecting dependency order (prerequisites before dependents).

**Why this priority**: Bulk updates are the main time-saver over manual package management.

**Independent Test**: Set up 3 packages with updates (one depends on another), run `update --all`, verify dependency order is respected.

**Acceptance Scenarios**:

1. **Given** 5 packages have updates and 2 have dependencies, **When** `update --all` runs, **Then** dependencies are installed before dependents
2. **Given** one package update fails, **When** the engine detects the failure, **Then** dependent packages are skipped but independent ones continue
3. **Given** `--allow-major` is NOT passed, **When** a major version upgrade is available, **Then** it is skipped with a notification

---

### User Story 3 - Dependency Resolution (Priority: P3)

A package requires ASCOM Platform 6.6+. The engine checks if the dependency is met, and if not, offers to install it first.

**Why this priority**: Dependency failures are the #1 install issue for astrophotography software.

**Independent Test**: Install a package that depends on ASCOM. Verify the engine checks and installs ASCOM first.

**Acceptance Scenarios**:

1. **Given** NINA requires ASCOM Platform 6.6+, **When** ASCOM 6.5 is installed, **Then** the engine offers to update ASCOM first
2. **Given** all dependencies are satisfied, **When** installing, **Then** the engine proceeds directly to the target package
3. **Given** a circular dependency (shouldn't happen), **When** detected, **Then** the engine reports the cycle and aborts

---

### User Story 4 - Update Policy Enforcement (Priority: P4)

The user configures update policies: "allow minor updates for all packages, require confirmation for major updates." The engine enforces these policies during bulk updates.

**Why this priority**: Users need control over what gets updated automatically, especially for critical imaging software.

**Independent Test**: Configure "minor only" policy, verify major updates are skipped. Configure per-package override, verify it takes effect.

**Acceptance Scenarios**:

1. **Given** a "minor only" default policy, **When** a major update is available, **Then** it is skipped with a notice
2. **Given** a per-package override allowing major updates for NINA, **When** a major update is available, **Then** NINA is updated but other packages respect the default policy

### Edge Cases

- Package in use during update: Report the process name and suggest closing it.
- Disk space insufficient: Check before downloading, report required vs available space.
- Power failure during install: On next launch, detect incomplete install and offer recovery.
- Downgrade request: Reject by default unless `--allow-downgrade` is explicitly passed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST orchestrate the update pipeline: check → download → backup → install → verify
- **FR-002**: System MUST resolve package dependencies and install them in topological order
- **FR-003**: System MUST enforce update policies (minor-only, major-allowed, manual, none) globally and per-package
- **FR-004**: System MUST support `--dry-run` mode that reports what would happen without making changes
- **FR-005**: System MUST support `--allow-major` flag to override minor-only policy for a single invocation
- **FR-006**: System MUST emit events for each pipeline step (check, download, backup, install, verify)
- **FR-007**: System MUST continue updating independent packages when one fails
- **FR-008**: System MUST skip dependent packages when a dependency fails
- **FR-009**: System MUST verify the installed version after installation (detect → compare)
- **FR-010**: System MUST support cancellation at any point in the pipeline
- **FR-011**: System MUST check available disk space before downloading
- **FR-012**: System MUST prevent downgrades unless explicitly requested

### Key Entities

- **UpdatePlan**: Ordered list of PackageUpdate items with resolved dependencies
- **PackageUpdate**: Package info, current version, target version, download URL, install config
- **UpdatePolicy**: Default policy + per-package overrides (Minor, Major, Manual, None)
- **PipelineEvent**: Enum of check/download/backup/install/verify events with progress data

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: End-to-end update for a single package completes in under 3 minutes (excluding download time)
- **SC-002**: Dependency resolution correctly orders all packages in the test catalog
- **SC-003**: Dry-run mode produces output identical to what a real run would do (except actual changes)
- **SC-004**: Policy enforcement correctly blocks/allows updates per configuration

## Assumptions

- Dependency information is declared in manifests (not discovered at runtime)
- The orchestration engine runs sequentially (one install at a time) to avoid conflicts
- Backup is optional per package — only runs if `[backup]` is defined in the manifest
- Depends on: spec 005 (catalog), spec 006+007 (detection), spec 008 (version checking), spec 010 (download), spec 011 (install), spec 013 (backup)

## Clarifications

- **Dependency graph is static**: Dependencies are declared in manifests (`[dependencies].requires`). No runtime dependency discovery. The graph is built from the catalog at plan time.
- **Topological sort with priority**: Sort by dependency order first, then by user-specified priority within the same dependency level. Circular dependencies are rejected at plan time.
- **Partial update plans**: If updating 5 packages and package 3 fails, packages 4-5 still run if they don't depend on package 3. The plan tracks which packages are independent.
- **Dry-run output format**: JSON-serializable plan showing: package, current_version, target_version, download_url, download_size, dependencies. Both CLI and GUI can display this.
- **Update policy inheritance**: Global policy from config (spec 004). Per-package overrides in config (`[updates.overrides.nina-app]`). CLI flags override both (`--allow-major`).
- **Verification step**: After install, re-run detection (spec 006/007) for the just-installed package. Compare detected version against expected. If mismatch, report "install succeeded but version doesn't match" (possible installer bug).
- **Backup integration**: If the package has `[backup]` in its manifest, run backup (spec 013) BEFORE install. If backup fails, ask user whether to proceed without backup.
- **Event sequence**: For each package: PlanCreated → CheckStarted → CheckComplete → DownloadStarted → DownloadProgress* → DownloadComplete → BackupStarted → BackupComplete → InstallStarted → InstallComplete → VerifyStarted → VerifyComplete.
