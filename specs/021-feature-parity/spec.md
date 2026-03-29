# Feature Specification: Feature Parity Verification

**Feature Branch**: `021-feature-parity`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 020 — verify Rust matches Go feature parity

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Feature Comparison Matrix (Priority: P1)

A maintainer runs the parity check and sees a matrix comparing every Go feature against its Rust equivalent. Each row shows: feature name, Go status (implemented/N/A), Rust status (implemented/partial/missing), and notes.

**Why this priority**: The matrix is the single source of truth for migration completeness. It gates the switch from Go to Rust.

**Independent Test**: Run the parity check, verify all features are categorized and no Go features are missing from the Rust column.

**Acceptance Scenarios**:

1. **Given** both Go and Rust codebases exist, **When** the parity check runs, **Then** a matrix lists all features with status
2. **Given** a feature is partially implemented in Rust, **When** viewing the matrix, **Then** it shows "partial" with notes on what's missing
3. **Given** all features are implemented, **When** the matrix is complete, **Then** every row shows "implemented" in the Rust column

---

### User Story 2 - Detection Parity (Priority: P2)

The parity check verifies that the Rust detection logic finds the same installed software (same packages, same versions) as the Go detection logic on the same machine.

**Why this priority**: Detection is the most critical parity requirement — users expect the same scan results.

**Independent Test**: Run both Go and Rust detection on a Windows machine with known software. Compare results.

**Acceptance Scenarios**:

1. **Given** NINA 3.1.2 is installed, **When** both Go and Rust detect it, **Then** both report version 3.1.2
2. **Given** 20 packages are installed, **When** both detect, **Then** the same 20 are found (no false positives or negatives)
3. **Given** a detection discrepancy, **When** flagged, **Then** the report shows the specific package and what differs

---

### User Story 3 - Performance Comparison (Priority: P3)

The parity check measures key performance metrics for both implementations: startup time, scan time, version check time, download speed.

**Why this priority**: The Rust version should be at least as fast as Go. Performance regression would be a migration blocker.

**Independent Test**: Benchmark both implementations on the same hardware, compare times.

**Acceptance Scenarios**:

1. **Given** both implementations, **When** benchmarked, **Then** Rust startup is faster or equal to Go
2. **Given** both implementations, **When** scanning 95 packages, **Then** Rust scan completes in equal or less time
3. **Given** a performance regression in Rust, **When** flagged, **Then** the report shows the specific operation and times

---

### User Story 4 - Manifest Compatibility (Priority: P4)

Both Go and Rust clients produce identical results when processing the same manifest data — same version resolution, same download URLs, same detection results.

**Why this priority**: Manifest compatibility ensures users can switch from Go to Rust without behavior changes.

**Independent Test**: Process 10 test manifests through both, compare outputs.

**Acceptance Scenarios**:

1. **Given** the same manifest TOML, **When** both parse it, **Then** the resulting Software struct has identical field values
2. **Given** the same version entry, **When** both resolve the download URL, **Then** the URLs are identical

### Edge Cases

- Go feature intentionally not ported (e.g., Bubble Tea TUI): Mark as "N/A — replaced by ratatui" in the matrix.
- Rust adds new features not in Go: Mark as "Rust-only" — parity is one-directional (Go→Rust).
- Version string parsing differs between Go and Rust: Flag as a discrepancy. The Rust parser should be a superset of Go's.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST generate a feature comparison matrix (Go vs Rust) covering all user-facing features
- **FR-002**: System MUST compare detection results between Go and Rust on the same machine
- **FR-003**: System MUST compare performance metrics (startup, scan, check, download) between implementations
- **FR-004**: System MUST verify manifest parsing compatibility (same input → same output)
- **FR-005**: System MUST flag discrepancies with specific details (package, field, Go value, Rust value)
- **FR-006**: System MUST support running parity checks on a Windows machine with real astrophotography software
- **FR-007**: System MUST produce a report suitable for go/no-go migration decision
- **FR-008**: System MUST categorize features as: implemented, partial, missing, N/A (intentionally not ported), Rust-only

### Key Entities

- **ParityMatrix**: Feature name → Go status × Rust status × notes
- **DetectionComparison**: Package ID → Go version × Rust version × match/mismatch
- **PerformanceBenchmark**: Operation → Go time × Rust time × ratio

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of Go features are either implemented in Rust or explicitly marked as N/A with justification
- **SC-002**: Detection parity: zero discrepancies on a test system with 20+ installed packages
- **SC-003**: Performance parity: Rust is equal or faster than Go on all measured operations
- **SC-004**: Manifest compatibility: zero parsing differences across all 95+ manifests

## Assumptions

- The Go codebase remains available for comparison (archived but not deleted)
- A Windows test machine with real astrophotography software is available for parity testing
- This is the final migration gate — passing this spec means the Rust version is ready to replace Go
- Depends on: all preceding specs (004-019)
