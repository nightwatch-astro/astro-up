# Feature Specification: CI Pipeline

**Feature Branch**: `018-ci-pipeline`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 017 — extend existing CI with integration tests and path filtering

## Note

This spec extends the existing CI from spec 001, not a replacement. The current CI has: check-rust (fmt, clippy, test), check-gui (Tauri clippy/test), check-frontend (lint, test, build), check-windows (check, test), and CI OK gate job.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Fast PR Feedback with Path Filtering (Priority: P1)

A developer opens a PR and only relevant CI jobs run. A Rust-only change skips frontend checks. A frontend-only change skips Windows tests. A docs-only change skips everything except the gate job.

**Why this priority**: Fast CI keeps development velocity. Running all jobs on every PR wastes time and runner minutes.

**Independent Test**: Open a frontend-only PR, verify Windows and Rust-only jobs are skipped.

**Acceptance Scenarios**:

1. **Given** a PR touching only `crates/`, **When** CI runs, **Then** Rust checks + Windows integration tests run. Frontend checks skip.
2. **Given** a PR touching only `frontend/`, **When** CI runs, **Then** frontend checks run. Rust checks and Windows tests skip.
3. **Given** a PR touching both `crates/` and `frontend/`, **When** CI runs, **Then** all jobs run in parallel.
4. **Given** a PR touching only `specs/` or `docs/`, **When** CI runs, **Then** only the gate job runs (trivially passes).
5. **Given** a PR touching `.github/workflows/`, **When** CI runs, **Then** all jobs run (CI changes affect everything).

---

### User Story 2 - Windows Integration Tests with Real Installs (Priority: P2)

On PRs touching Rust code, CI runs integration tests on a Windows runner that exercise real detection and installation against actual software (PHD2 via InnoSetup, ASCOM Platform via MSI).

**Why this priority**: Detection and installation are Windows-specific. Must be tested against real installers, real registry entries, real PE files.

**Independent Test**: PR touching detection code triggers Windows tests that install PHD2, detect it, verify version, uninstall.

**Acceptance Scenarios**:

1. **Given** a PR touching `crates/`, **When** Windows integration tests run, **Then** PHD2 is downloaded and silently installed
2. **Given** PHD2 is installed, **When** registry detection runs, **Then** the correct version is found in the registry
3. **Given** PHD2 is installed, **When** PE detection runs on phd2.exe, **Then** the correct file version is extracted
4. **Given** detection passes, **When** cleanup runs, **Then** PHD2 is silently uninstalled
5. **Given** ASCOM Platform is downloaded, **When** MSI silent install runs, **Then** ASCOM registers in the registry and ASCOM Profile keys are created

---

### User Story 3 - Single Gate Job for Branch Protection (Priority: P3)

A single "CI OK" job aggregates all other job results. Branch protection requires only this one check. Adding or removing CI jobs doesn't require updating branch protection.

**Why this priority**: Simplifies branch protection management. One check to rule them all.

**Independent Test**: All jobs pass → CI OK passes. One job fails → CI OK fails. Skipped jobs don't block.

**Acceptance Scenarios**:

1. **Given** all required jobs pass, **When** the gate job evaluates, **Then** CI OK is green
2. **Given** one job fails, **When** the gate job evaluates, **Then** CI OK is red
3. **Given** some jobs are skipped (path filtering), **When** the gate job evaluates, **Then** skipped jobs don't block — CI OK is green if all non-skipped jobs pass

---

### User Story 4 - Conventional Commit Enforcement (Priority: P4)

PR titles are validated against conventional commit format. Non-conforming titles fail CI.

**Why this priority**: Conventional commits drive release-plz changelog generation and version bumps.

**Acceptance Scenarios**:

1. **Given** a PR titled `feat: add download manager`, **When** CI runs, **Then** the PR title check passes
2. **Given** a PR titled `added download stuff`, **When** CI runs, **Then** the PR title check fails with a suggestion

### Edge Cases

- CI runner cache stale: Cargo and pnpm caches keyed by lockfile hash. Stale cache = cache miss, not failure.
- Windows runner unavailable: CI OK reports the Windows job as failed, not skipped. Alerts the team.
- Flaky tests: Tests must be deterministic. Flaky = bug, not CI config issue.
- Dependabot PRs: Auto-merge workflow (separate) handles these. CI runs normally.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: CI MUST use `dorny/paths-filter@v4` to skip irrelevant jobs based on changed files
- **FR-002**: CI MUST define path groups: `crates` (crates/**), `frontend` (frontend/**), `ci` (.github/**), `docs` (specs/**, docs/**, *.md)
- **FR-003**: CI MUST run Rust checks (fmt, clippy, test) on `crates` changes
- **FR-004**: CI MUST run frontend checks (lint, type-check, test, build) on `frontend` changes
- **FR-005**: CI MUST run Tauri GUI checks (clippy with Tauri system deps) on `crates/astro-up-gui/**` changes
- **FR-006**: CI MUST run Windows integration tests on `crates` changes
- **FR-007**: CI MUST install PHD2 (InnoSetup) and ASCOM Platform (MSI) on the Windows runner for integration tests
- **FR-008**: CI MUST test detection (registry + PE) against the real installed software
- **FR-009**: CI MUST clean up (uninstall) test software after integration tests
- **FR-010**: CI MUST provide a single "CI OK" gate job that branch protection requires
- **FR-011**: CI MUST validate PR titles against conventional commit format (cocogitto or semantic-pull-request action)
- **FR-012**: CI MUST cache Cargo registry/target (Swatinem/rust-cache) and pnpm store between runs
- **FR-013**: CI MUST run Rust and frontend checks in parallel when both are triggered
- **FR-014**: CI MUST install Tauri system dependencies on Ubuntu for GUI crate checks
- **FR-015**: CI MUST run on both push to main and pull_request events
- **FR-016**: CI MUST use concurrency groups to cancel redundant runs on the same PR
- **FR-017**: CI MUST reuse shared workflows from nightwatch-astro/.github where applicable

### Path Filter Configuration

| Path Group | Triggers |
|------------|----------|
| `crates` | check-rust, check-gui, check-windows (integration) |
| `frontend` | check-frontend |
| `ci` | all jobs (workflow changes affect everything) |
| `docs` | gate job only |

### Integration Test Matrix

| Test | Installer Type | What it validates |
|------|---------------|-------------------|
| PHD2 install + detect | InnoSetup | Silent install, registry detection, PE version, silent uninstall |
| ASCOM Platform install + detect | MSI | MSI install, ASCOM Profile registry, MSI uninstall |
| ZIP extraction | ZIP (test fixture) | Extract, zip-slip guard, file verification |
| Version comparison | N/A | Semver, date, custom format parsing and ordering |
| Catalog load | N/A | SQLite open, FTS5 search, signature verify (test fixtures) |

### Key Entities

- **CI Gate Job**: Single required check aggregating all job results
- **PathFilter**: dorny/paths-filter output determining which jobs run
- **IntegrationTestFixture**: Test installers + expected detection results

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Frontend-only PR CI completes in under 2 minutes
- **SC-002**: Rust-only PR CI completes in under 5 minutes (including Windows integration)
- **SC-003**: Full-stack PR CI completes in under 5 minutes (parallel jobs)
- **SC-004**: Cache hit rate above 80% for Cargo and pnpm stores
- **SC-005**: Windows integration tests pass for both InnoSetup and MSI install methods

## Assumptions

- GitHub Actions is the CI platform
- Ubuntu runners for Rust + frontend checks, Windows runners for integration tests
- GitHub Free tier — no custom runner images, standard GitHub-hosted runners only
- Extends existing CI from spec 001 — not a rewrite
- PHD2 and ASCOM Platform are stable enough to use as CI test subjects (unlikely to break between runs)
- Depends on: spec 001 (existing CI), spec 006 (detection logic to test), spec 011 (install logic to test)
