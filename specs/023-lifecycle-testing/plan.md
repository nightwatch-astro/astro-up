# Implementation Plan: 023-lifecycle-testing

**Branch**: `023-lifecycle-testing` | **Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/023-lifecycle-testing/spec.md`

## Summary

Automated package lifecycle testing workflow that installs packages on Windows CI runners, discovers detection signatures by probing all 7 detection methods blind, and submits the discovered `[detection]` TOML config as PRs to the manifests repo. Also expands the catalog detection schema to support all DetectionConfig fields, and records install paths in the ledger after every install.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: pelite 0.10 (PE parsing, existing), winreg (registry, existing), wmi (WMI, existing), reqwest (downloads, existing), toml 0.9 (manifest reading, promote from dev-dep), serde_json (fallback serialization, existing)
**Storage**: SQLite via rusqlite (catalog detection table expansion)
**Testing**: cargo test + insta (snapshot testing for TOML output, JSON report)
**Target Platform**: Windows (runtime), cross-platform (compilation, dry-run PE inspection)
**Project Type**: CLI tool + CI workflow
**Performance Goals**: 10-minute timeout per package lifecycle test
**Constraints**: Windows-only for install/uninstall/registry probing; cross-platform for compilation and dry-run
**Scale/Scope**: ~96 packages in manifests repo, ~10 with existing hand-written detection configs

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | New modules in astro-up-core: `detect/discovery.rs`, `lifecycle.rs`, `catalog/manifest.rs`. No new crates. |
| II. Platform Awareness | PASS | Discovery and lifecycle probing are `#[cfg(windows)]`. PE inspection is cross-platform via pelite. CLI compiles on all platforms. |
| III. Test-First | PASS | Integration tests for discovery (Windows-only), snapshot tests for TOML output and JSON report (cross-platform). |
| IV. Thin Tauri Boundary | PASS | No GUI changes. All logic in astro-up-core, CLI is thin adapter. |
| V. Spec-Driven | PASS | This spec drives the implementation. |
| VI. Simplicity | PASS | Reuses existing install/detect/download pipelines. No new abstractions beyond discovery scanner and lifecycle runner. |

## Project Structure

### Documentation (this feature)

```text
specs/023-lifecycle-testing/
├── spec.md
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── discovery.rs
│   ├── lifecycle.rs
│   └── manifest-reader.rs
├── checklists/
│   ├── requirements.md
│   └── lifecycle-ci.md
└── tasks.md             # Phase 2 output (speckit.tasks)
```

### Source Code (repository root)

```text
crates/astro-up-core/src/
├── detect/
│   ├── discovery.rs     # NEW: Blind detection probing (all 7 methods)
│   └── mod.rs           # MODIFY: Add pub mod discovery
├── lifecycle.rs          # NEW: Lifecycle test runner (phases, report)
├── catalog/
│   ├── manifest.rs      # NEW: TOML manifest reader
│   ├── reader.rs        # MODIFY: Expand detection_config() for new columns
│   └── mod.rs           # MODIFY: Add pub mod manifest
├── install/
│   └── mod.rs           # MODIFY: Record install_path in ledger after install
└── types/
    └── detection.rs     # No changes (already has all fields)

crates/astro-up-cli/src/
├── commands/
│   ├── lifecycle_test.rs # NEW: lifecycle-test subcommand
│   └── mod.rs           # MODIFY: Add pub mod lifecycle_test
└── lib.rs               # MODIFY: Add LifecycleTest variant to Commands enum

.github/workflows/
└── lifecycle-test.yml   # NEW: GitHub Actions workflow

# Cross-repo: nightwatch-astro/astro-up-manifests
crates/compiler/src/
├── schema.rs            # MODIFY: Update detection table DDL
└── compile.rs           # MODIFY: Read all detection fields, serialize fallback as JSON

tests/
└── create_fixture_catalog.rs  # MODIFY: Update detection table in test fixture
```

**Structure Decision**: All new code lives in existing crates as new modules. No new crates needed. Cross-repo compiler changes are minimal (schema DDL + column mapping).

## Implementation Phases

### Phase A: Foundation (core modules)

1. **TOML manifest reader** (`catalog/manifest.rs`)
   - `ManifestReader::read()` — deserialize TOML to `Software`
   - `ManifestReader::read_by_id()` — lookup by package ID
   - `ManifestReader::list_missing_detection()` — scan for packages needing discovery
   - Tests: read sample manifests, verify deserialization

2. **Detection discovery module** (`detect/discovery.rs`)
   - `DiscoveryScanner::discover()` — probe all 7 methods blind
   - Registry discovery: enumerate uninstall keys, match by manifest `name` (primary) / package ID (fallback)
   - PE discovery: scan InstallLocation + common program dirs for .exe files, read PE headers
   - FileExists/ConfigFile/ASCOM/WMI/DriverStore: probe known locations
   - `build_config()` — generate DetectionConfig from ranked candidates with fallback chain (max depth 3)
   - Path tokenization: convert absolute paths to `{program_files}` tokens
   - Tests: Windows-only integration tests with known paths

3. **Lifecycle runner** (`lifecycle.rs`)
   - Phase orchestration: download → install → detect → verify-install → uninstall → verify-removal → report
   - `resolve_latest_version()` — read from `versions/{id}/` directory
   - `resolve_download_url()` — substitute `$version` in autoupdate URL
   - `config_to_toml()` — serialize DetectionConfig to TOML string
   - Timeout handling, cleanup on failure
   - Dry-run mode: skip install/uninstall phases
   - download_only handling: require install_dir, skip install/uninstall
   - Tests: snapshot tests for report JSON and TOML output

### Phase B: Schema & Integration

4. **Catalog detection table expansion**
   - **Reader** (`catalog/reader.rs`): update `detection_config()` SQL to read new columns, deserialize `fallback_config` JSON blob
   - **Test fixture** (`create_fixture_catalog.rs`): update detection table DDL
   - **Compiler** (manifests repo): update `schema.rs` DDL, update `compile.rs` to read all TOML fields and serialize fallback as JSON
   - Tests: round-trip test (TOML → compile → SQLite → read → DetectionConfig)

5. **Install path ledger recording**
   - After install succeeds in `InstallerService::install()`, run detection chain
   - Extract `install_path` from `DetectionResult::Installed { install_path }`
   - Fallback: registry InstallLocation → default dir
   - Store in `LedgerEntry.install_path`
   - Tests: install mock package, verify ledger has path

### Phase C: CLI & Workflow

6. **CLI subcommand** (`commands/lifecycle_test.rs`)
   - `LifecycleTest` variant in `Commands` enum
   - Args: `<package_id>`, `--manifest-path`, `--version`, `--install-dir`, `--dry-run`, `--json`
   - Exit codes: 0/1/2/3/4 per FR-032
   - Human-readable output (phase table) and JSON mode
   - GitHub Actions job summary output (detect `GITHUB_STEP_SUMMARY` env var)

7. **GitHub Actions workflow** (`lifecycle-test.yml`)
   - workflow_dispatch: `package_id` (required), `version` (optional), `dry_run` (boolean)
   - Two-job pattern for matrix sweep: prepare (scan manifests) → test (matrix)
   - Checkout both repos (astro-up + manifests)
   - App token for cross-repo PR creation
   - Build CLI binary, run lifecycle-test
   - Cleanup step with `if: always()` + `continue-on-error: true`
   - Job summary + JSON artifact upload
   - Auto-create PR with `gh pr create` (or force-push + update existing)
   - Max parallel: 5

## Cross-Repo Coordination

The manifests repo compiler changes are small and self-contained:

1. **schema.rs**: Update detection table CREATE TABLE statement (add columns, replace fallback_method/fallback_path with fallback_config)
2. **compile.rs**: Update the function that reads TOML `[detection]` sections to map all fields, serialize fallback chain as JSON blob

These changes can be made in a separate PR against `nightwatch-astro/astro-up-manifests` and merged before or after the main repo changes (the reader handles missing columns gracefully).

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| Install hangs on CI | 10-min timeout per phase, `kill_on_drop`, always-run cleanup |
| Discovery finds wrong package | Manifest `name` matching (not just slug), confidence ranking, maintainer reviews PR |
| Uninstall fails, dirty runner | Ephemeral runners, 3-tier cleanup (uninstall → kill → report) |
| False positive registry matches | Case-insensitive substring on product name, not slug |
| Manifests repo token permissions | Same app token pattern as release.yml (proven) |

## Complexity Tracking

No constitution violations to justify.
