# Implementation Plan: Software and Driver Detection

**Branch**: `006-registry-pe-detection` | **Date**: 2026-03-30 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/006-registry-pe-detection/spec.md`

## Summary

Implement a detection system that scans all catalog packages to determine which astrophotography software and drivers are installed on the user's Windows machine. Uses a fallback chain (registry → PE file → file_exists) per package, with specialized methods for WMI driver detection and ASCOM Profile. Detected packages are auto-persisted as `Acknowledged` ledger entries. Includes brownfield hardware discovery via VID:PID matching and an in-memory detection cache with event-driven invalidation.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: pelite 0.10 (PE), winreg 0.56 (registry), wmi 0.14 (WMI/hardware), directories 6.0 (platform paths)
**Storage**: SQLite (existing — ledger entries for Acknowledged packages)
**Testing**: cargo test, insta (snapshots), tempfile, test PE fixtures
**Target Platform**: Windows primary; macOS/Linux for CI (cross-platform PE + stubs)
**Project Type**: Library (astro-up-core module)
**Performance Goals**: Full scan <5s for ~95 packages, cached lookups <1ms
**Constraints**: Windows-only APIs behind `cfg(windows)`, WMI timeout 10s per query
**Scale/Scope**: ~95 catalog packages, ~10-15 typically installed

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | All code in `astro-up-core/src/detect/` module, no new crate |
| II. Platform Awareness | PASS | `cfg(windows)` on registry/wmi/ascom modules, stubs on other platforms. pelite cross-platform. |
| III. Test-First | PASS | Unit tests for chain logic/parsing, integration for PE files, Windows CI for registry/WMI |
| IV. Thin Tauri Boundary | PASS | All logic in core, GUI/CLI call `DetectionService` trait |
| V. Spec-Driven | PASS | Full spec with 20 FRs, 5 SCs, clarifications, checklist review |
| VI. Simplicity | PASS | One module, standard fallback chain pattern, no speculative abstractions |

**Post-design re-check**: All principles still satisfied. No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/006-registry-pe-detection/
├── spec.md
├── decisions.md
├── plan.md              # This file
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── detect-trait.rs
├── checklists/
│   ├── requirements.md
│   └── detection.md
└── tasks.md             # Next step (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/astro-up-core/src/
├── detect/
│   ├── mod.rs           # DetectionService impl, scan orchestration, chain runner
│   ├── registry.rs      # cfg(windows) — uninstall registry key detection
│   ├── pe.rs            # PE file version extraction via pelite (cross-platform)
│   ├── wmi_driver.rs    # cfg(windows) — WMI Win32_PnPSignedDriver queries
│   ├── ascom.rs         # cfg(windows) — ASCOM Profile registry detection
│   ├── file.rs          # file_exists + config_file detection methods
│   ├── cache.rs         # In-memory DetectionCache with event-driven invalidation
│   ├── hardware.rs      # cfg(windows) — VID:PID matching via Win32_PnPEntity
│   └── path.rs          # Shared path token resolver ({program_files} → real path)
├── types/
│   └── detection.rs     # Existing — DetectionConfig, DetectionMethod (extend if needed)
├── ledger.rs            # Existing — add Acknowledged entry management
└── lib.rs               # Add `pub mod detect;`

crates/astro-up-core/
├── Cargo.toml           # Add pelite, winreg (cfg windows), wmi (cfg windows)
└── tests/
    ├── detect_chain.rs  # Chain fallback logic integration tests
    ├── detect_pe.rs     # PE version extraction with test fixture
    └── fixtures/
        └── test.exe     # Minimal PE with known VS_FIXEDFILEINFO
```

**Structure Decision**: Single `detect/` module inside `astro-up-core` per Constitution Principle I. All detection methods as submodules. Shared path resolver also here since detection is the first (and currently only) consumer.

## Implementation Phases

### Phase A: Foundation (path resolver + types + chain runner)
- Path token resolver utility
- Extend DetectionResult/DetectionMethod types
- Chain runner logic (fallback, stop-at-first-success)
- Unit tests for chain logic and path expansion

### Phase B: Core Detection Methods (registry + PE)
- Registry detection (`winreg`, both views, HKLM+HKCU)
- PE file detection (`pelite`, cross-platform)
- `file_exists` and `config_file` methods
- Integration tests with PE fixture, snapshot tests

### Phase C: Windows-Specific Methods (WMI + ASCOM)
- WMI driver detection (`Win32_PnPSignedDriver`)
- ASCOM Profile detection (registry-based)
- WMI timeout enforcement (10s)
- Windows CI tests

### Phase D: Scan Orchestration + Ledger Integration
- Full scan: load catalog → run chain per package → collect results
- Ledger sync: insert/update/remove Acknowledged entries
- Per-package error collection (non-fatal)
- Detection cache with invalidation API

### Phase E: Hardware Discovery
- VID:PID pattern parsing and matching
- `Win32_PnPEntity` WMI query for connected devices
- Cross-reference with detection results (skip already-managed)
- Wildcard matching tests

## Dependencies

```
Phase A ──→ Phase B ──→ Phase D
Phase A ──→ Phase C ──→ Phase D
                        Phase D ──→ Phase E
```

Phase B and C can run in parallel after A. Phase D needs both. Phase E needs D (for already-managed check).

## Complexity Tracking

No constitution violations — no entries needed.
