# Feature Specification: CI Pipeline

**Feature Branch**: `018-ci-pipeline`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 017 — GitHub Actions CI

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Pull Request Validation (Priority: P1)

A developer opens a PR and CI automatically runs Rust checks (fmt, clippy, test), frontend checks (lint, test, build), and conventional commit validation. A single "CI OK" gate job reports pass/fail for branch protection.

**Why this priority**: PR validation is the quality gate that prevents regressions on main.

**Independent Test**: Open a PR with a deliberate lint error, verify CI catches it.

**Acceptance Scenarios**:

1. **Given** a PR with clean code, **When** CI runs, **Then** all checks pass and "CI OK" is green
2. **Given** a PR with a clippy warning, **When** CI runs, **Then** the Rust check fails with the warning
3. **Given** a PR with a non-conventional commit title, **When** CI runs, **Then** the PR title check fails

---

### User Story 2 - Windows Integration Tests (Priority: P2)

On PRs that touch Rust code, CI runs integration tests on a Windows runner that exercise real detection (registry, PE, WMI) against pre-installed test software.

**Why this priority**: Detection and installation logic is Windows-specific. Must be tested on real Windows.

**Independent Test**: Merge a detection change, verify Windows tests pass against real registry entries.

**Acceptance Scenarios**:

1. **Given** a PR touching `crates/` code, **When** CI runs on Windows, **Then** integration tests run against real Windows APIs
2. **Given** a PR touching only `frontend/`, **When** CI evaluates, **Then** Windows tests are skipped (path filtering)

---

### User Story 3 - Fast Feedback Loop (Priority: P3)

CI is structured for fast feedback: Rust checks run in parallel with frontend checks. Path filtering skips irrelevant jobs. The typical PR gets feedback in under 3 minutes.

**Why this priority**: Slow CI kills developer productivity. Fast feedback keeps momentum.

**Independent Test**: Open a frontend-only PR, verify only frontend jobs run. Check total time.

**Acceptance Scenarios**:

1. **Given** a Rust-only PR, **When** CI runs, **Then** frontend checks are skipped
2. **Given** a full-stack PR, **When** CI runs, **Then** Rust and frontend checks run in parallel
3. **Given** all checks pass, **When** the gate job evaluates, **Then** total CI time is under 5 minutes

### Edge Cases

- CI runner cache is stale: Cargo and pnpm caches are keyed by lockfile hash. Stale cache is a cache miss, not a failure.
- Tauri build fails on Windows due to missing system deps: Ubuntu jobs don't build Tauri. Windows job installs deps via choco/winget.
- Flaky test: Tests must be deterministic. Flaky tests are bugs, not CI configuration issues.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: CI MUST run `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` for Rust
- **FR-002**: CI MUST run `pnpm lint`, `pnpm test` (Vitest), `pnpm build` for frontend
- **FR-003**: CI MUST validate PR titles against conventional commit format
- **FR-004**: CI MUST use path filtering to skip irrelevant jobs (dorny/paths-filter)
- **FR-005**: CI MUST provide a single "CI OK" gate job that branch protection requires
- **FR-006**: CI MUST run Windows integration tests on PRs touching Rust code
- **FR-007**: CI MUST cache Cargo registry/target and pnpm store between runs
- **FR-008**: CI MUST run Rust and frontend checks in parallel
- **FR-009**: CI MUST install Tauri system dependencies on Ubuntu for GUI crate checks
- **FR-010**: CI MUST reuse shared workflows from nightwatch-astro/.github where applicable

### Key Entities

- **CI Gate Job**: Single required check that aggregates all other job results
- **Path Filter**: dorny/paths-filter determining which jobs to run based on changed files

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Typical PR CI completes in under 5 minutes
- **SC-002**: Path filtering correctly skips irrelevant jobs (measured by job skip rate)
- **SC-003**: Cache hit rate above 80% for Cargo and pnpm stores
- **SC-004**: Zero false negatives — CI never passes code that would fail locally

## Assumptions

- GitHub Actions is the CI platform (not GitLab CI, not Jenkins)
- Ubuntu runners for Rust + frontend checks, Windows runners for integration tests
- The existing CI from spec 001 is the starting point — this spec extends it
- Depends on: spec 001 (existing CI scaffolding)
