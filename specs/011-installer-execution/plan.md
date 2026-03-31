# Implementation Plan: Installer Execution

**Branch**: `011-installer-execution` | **Date**: 2026-03-30 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/011-installer-execution/spec.md`

## Summary

Implement the installer execution subsystem: silent installation of 10 installer types on Windows with exit code interpretation, admin elevation (sudo/runas), ZIP extraction with zip-slip protection, process tree management via Job Objects, pre/post hooks, uninstall support, and install ledger recording. Two-tier execution strategy: `tokio::process::Command` for simple installers, `CreateProcessW` + Job Objects for bootstrappers.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+)
**Primary Dependencies**: tokio 1 (+ process feature), zip 2, windows 0.62 (cfg(windows)), tokio-util 0.7 (CancellationToken, existing)
**Storage**: SQLite (existing, ledger entries via rusqlite)
**Testing**: cargo test, insta (snapshots), tempfile (ZIP fixtures)
**Target Platform**: Windows (primary), macOS/Linux (CI compilation + cross-platform tests)
**Project Type**: Library (astro-up-core crate)
**Performance Goals**: SC-001 silent install works for all types; SC-004 timeout prevents hangs
**Constraints**: Windows-only execution gated behind `cfg(windows)`; cross-platform CI must compile and run unit tests
**Scale/Scope**: 10 installer types, 20 FRs, 7 edge cases

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | New `install/` module in astro-up-core, no new crate |
| II. Platform Awareness | PASS | Windows code gated with `cfg(windows)`, `windows` crate as optional dep |
| III. Test-First | PASS | Unit tests for switches/exit codes (cross-platform), integration for ZIP, `cfg(windows)` for process tests |
| IV. Thin Tauri Boundary | PASS | All logic in core, GUI/CLI are consumers |
| V. Spec-Driven | PASS | Full speckit pipeline |
| VI. Simplicity | PASS | Two-tier execution (simple + job object) only when needed; no speculative abstractions |

No violations. No complexity tracking needed.

## Project Structure

### Documentation (this feature)

```text
specs/011-installer-execution/
  spec.md
  decisions.md
  plan.md              # This file
  research.md
  data-model.md
  quickstart.md
  contracts/
    installer-service.rs
  checklists/
    requirements.md
  tasks.md             # Next step (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/astro-up-core/src/
  install/
    mod.rs            # InstallerService, Installer trait impl, orchestration
    types.rs          # InstallRequest, InstallResult, ExitCodeOutcome
    switches.rs       # Default silent switches per InstallMethod
    exit_codes.rs     # Exit code interpretation + semantic mapping
    zip.rs            # ZIP extraction, zip-slip guard, single-root flattening
    hooks.rs          # Pre/post install hook execution
    elevation.rs      # [cfg(windows)] is_elevated(), sudo detection, runas
    process.rs        # [cfg(windows)] spawn, Job Objects, timeout
    uninstall.rs      # [cfg(windows)] registry lookup, silent uninstall
    ledger.rs         # Record LedgerEntry after install
  traits.rs             # Modified: Installer trait returns Result<InstallResult, CoreError>
  events.rs             # Modified: add InstallFailed, InstallRebootRequired
  types/install.rs      # Modified: add timeout field to InstallConfig
  ledger.rs             # Modified: add install_path field to LedgerEntry
  lib.rs                # Modified: add pub mod install

crates/astro-up-core/tests/
  install_switches.rs       # Cross-platform: switch resolution
  install_exit_codes.rs     # Cross-platform: exit code mapping
  install_zip.rs            # Cross-platform: ZIP extraction + zip-slip
  install_integration.rs    # cfg(windows): process execution
```

**Structure Decision**: Module within existing `astro-up-core` crate per Constitution Principle I. 10 files in `install/` matching distinct FR groups. Cross-platform logic separated from `cfg(windows)` code.

## Implementation Phases

### Phase A: Foundation (cross-platform)

Types, switches, exit codes, event variants, config field changes. All testable on Ubuntu CI.

- Create `install/types.rs` with `InstallRequest`, `InstallResult`, `ExitCodeOutcome`
- Create `install/switches.rs` with default switch tables per `InstallMethod` and resolution logic
- Create `install/exit_codes.rs` with `interpret_exit_code()` using precedence: success_codes > known_exit_codes > defaults
- Add `timeout: Option<Duration>` to `InstallConfig` (with humantime-serde)
- Add `install_path: Option<PathBuf>` to `LedgerEntry`
- Add `InstallFailed`, `InstallRebootRequired` event variants
- Update `Installer` trait signature to `Result<InstallResult, CoreError>`
- Update snapshot tests for modified types
- Wire `pub mod install` in lib.rs

### Phase B: ZIP extraction (cross-platform)

- Add `zip = "2"` dependency
- Create `install/zip.rs` with `extract_zip()`:
  - Use `enclosed_name()` for zip-slip protection (reject if None)
  - Detect single-root-directory archives (scan all entry prefixes)
  - Strip common root when single-root detected
  - Wrap in `spawn_blocking` (zip crate is sync)
- Integration tests with crafted ZIP fixtures (normal, malicious, single-root, multi-root, empty)

### Phase C: Process execution (Windows-only)

- Add `windows = "0.62"` with feature flags and tokio `"process"` feature
- Create `install/process.rs`:
  - `spawn_simple()` via `tokio::process::Command` + timeout + CancellationToken via `select!`
  - `spawn_with_job_object()` via `CreateProcessW` + `CREATE_SUSPENDED` + Job Object + `spawn_blocking(WaitForSingleObject)`
- Create `install/elevation.rs`:
  - `is_elevated()` via `IsUserAnAdmin`
  - `elevate_and_reexec()` detects `sudo.exe` on PATH, falls back to `ShellExecuteExW runas`
- Create `install/mod.rs` orchestration:
  - Pre-hook, elevation check, spawn (simple or job object based on method), interpret exit code, post-hook, ledger
  - Reactive elevation retry on exit code 740
- `cfg(windows)` integration tests

### Phase D: Hooks, uninstall, ledger

- Create `install/hooks.rs`: detect `.ps1` for PowerShell, else `cmd /c`. 60s timeout. Hooks inherit elevation.
- Create `install/uninstall.rs` (`cfg(windows)`):
  - `find_uninstall_command()`: read registry `QuietUninstallString` or `UninstallString`
  - Silent uninstall with appropriate switches
  - ZIP/portable: delete directory (with confirmation via event/callback)
- Create `install/ledger.rs`: write `LedgerEntry` with `install_path` after success
- Wire `upgrade_behavior = "uninstall_previous"` flow: uninstall current then install new

### Phase E: Integration + polish

- Implement `Installer` trait for `InstallerService`
- `DownloadOnly` handling: open containing folder via `ShellExecuteW open` on Windows
- `#[cfg(not(windows))]` stub returning unsupported platform error
- Metric recording (`INSTALL_DURATION_SECONDS`)
- End-to-end test: download fixture, install, verify ledger entry
- Cancellation test: start install, cancel, verify cleanup

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Execution tiers | Simple (tokio::process) + Job Object (windows crate) | Most installers are single-process; Job Objects only for bootstrappers |
| ZIP-slip protection | `enclosed_name()` API | Built into zip crate, canonical defense |
| Elevation | sudo.exe then ShellExecuteExW runas | Modern Windows first, universal fallback |
| Switch resolution | Empty = no switches, missing = defaults | Unambiguous, manifest author controls |
| Exit code precedence | success_codes > known_exit_codes > defaults | success_codes is the authoritative list |
| Hook security | Trusted (from signed manifests) | Same trust as running the installer |
| InstallResult | Enum: Success/SuccessRebootRequired/Cancelled | Reboot is success state, not error |

## Cross-Spec Impact

| Spec | Change | Compatibility |
|------|--------|---------------|
| 003 (types) | Installer trait: `Result<()>` to `Result<InstallResult>` | Breaking, no downstream consumers yet |
| 003 (types) | LedgerEntry: add `install_path: Option<PathBuf>` | Additive, backward compatible |
| 003 (types) | InstallConfig: add `timeout: Option<Duration>` | Additive, backward compatible |
| 006 (detection) | Unblocks #215 (PE detection from ledger paths) | Deferred issue resolved |
| 012 (orchestration) | Consumes InstallerService via Installer trait | Forward dependency, no changes needed |
