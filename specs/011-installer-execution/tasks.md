# Tasks: Installer Execution

**Input**: Design documents from `/specs/011-installer-execution/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/installer-service.rs, research.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US6)
- Paths relative to `crates/astro-up-core/`

## Phase 1: Setup

**Purpose**: Add dependencies and wire module structure

- [ ] T001 Add `zip = "2"` to `crates/astro-up-core/Cargo.toml` dependencies
- [ ] T002 Add `windows = { version = "0.62", features = ["Win32_System_JobObjects", "Win32_System_Threading", "Win32_UI_Shell", "Win32_Security"] }` to `[target.'cfg(windows)'.dependencies]` in `crates/astro-up-core/Cargo.toml`
- [ ] T003 Add `"process"` feature to existing tokio dependency in `crates/astro-up-core/Cargo.toml`
- [ ] T004 Create `crates/astro-up-core/src/install/mod.rs` with module declarations and `InstallerService` struct skeleton
- [ ] T005 Wire `pub mod install;` in `crates/astro-up-core/src/lib.rs`

---

## Phase 2: Foundational (Cross-spec type changes)

**Purpose**: Modify shared types that all user stories depend on. MUST complete before any US work.

- [ ] T006 Add `install_path: Option<PathBuf>` field to `LedgerEntry` in `crates/astro-up-core/src/ledger.rs` and update serialization and tests
- [ ] T007 [P] Add `timeout: Option<Duration>` field (with `humantime-serde`) to `InstallConfig` in `crates/astro-up-core/src/types/install.rs` and update serialization and tests
- [ ] T008 [P] Create `InstallResult` enum (Success, SuccessRebootRequired, Cancelled) and `ExitCodeOutcome` enum in `crates/astro-up-core/src/install/types.rs`
- [ ] T009 [P] Create `InstallRequest` and `UninstallRequest` structs in `crates/astro-up-core/src/install/types.rs`
- [ ] T010 Update `Installer` trait in `crates/astro-up-core/src/traits.rs`: change return type to `Result<InstallResult, CoreError>`, add `uninstall()` method, update `InstallOptions` to accept `InstallRequest`
- [ ] T011 [P] Add `InstallFailed { id: String, error: String }` and `InstallRebootRequired { id: String }` event variants to `crates/astro-up-core/src/events.rs`
- [ ] T012 Update snapshot tests for events and types in `crates/astro-up-core/src/snapshots/` and `crates/astro-up-core/tests/`

**Checkpoint**: All shared types updated, `cargo check` passes, snapshot tests updated

---

## Phase 3: User Story 1 - Silent Installation (Priority: P1)

**Goal**: Run installers silently using type-appropriate default switches. Custom manifest switches fully replace defaults.

**Independent Test**: Build switch resolution for each of the 10 installer types, verify correct args generated.

- [ ] T013 [P] [US1] Create default silent switch table for all 10 `InstallMethod` variants in `crates/astro-up-core/src/install/switches.rs`
- [ ] T014 [P] [US1] Implement `resolve_switches(config: &InstallConfig) -> Vec<String>` in `crates/astro-up-core/src/install/switches.rs` with empty-means-none and missing-means-defaults logic
- [ ] T015 [P] [US1] Write unit tests for switch resolution in `crates/astro-up-core/tests/install_switches.rs` covering all 10 types, custom override, empty override, missing section
- [ ] T016 [US1] Implement simple process spawning via `tokio::process::Command` with timeout and CancellationToken in `crates/astro-up-core/src/install/process.rs` (cfg(windows) with cfg(not(windows)) stub)
- [ ] T017 [US1] Implement MSI invocation path (msiexec with switches) in `crates/astro-up-core/src/install/process.rs`
- [ ] T018 [US1] Wire `InstallerService::install()` orchestration in `crates/astro-up-core/src/install/mod.rs` to spawn process with resolved switches, emit InstallStarted and InstallComplete events
- [ ] T019 [US1] Implement `Installer` trait for `InstallerService` in `crates/astro-up-core/src/install/mod.rs` with cfg(not(windows)) returning unsupported error

**Checkpoint**: Silent installation works for all exe-based installer types

---

## Phase 4: User Story 2 - Exit Code Interpretation (Priority: P2)

**Goal**: Map exit codes to semantic meanings using manifest known_exit_codes table.

**Independent Test**: Unit test exit code resolution for all KnownExitCode variants, verify precedence logic.

- [ ] T020 [P] [US2] Implement `interpret_exit_code(code: i32, config: &InstallConfig) -> ExitCodeOutcome` in `crates/astro-up-core/src/install/exit_codes.rs` with precedence: success_codes then known_exit_codes then defaults (740=elevation, 3010=reboot)
- [ ] T021 [P] [US2] Write unit tests for exit code interpretation in `crates/astro-up-core/tests/install_exit_codes.rs` covering all KnownExitCode variants, precedence, success_codes overlap, unknown codes
- [ ] T022 [US2] Wire exit code interpretation into `InstallerService::install()` in `crates/astro-up-core/src/install/mod.rs` to map ExitCodeOutcome to InstallResult or CoreError

**Checkpoint**: Exit codes correctly interpreted, InstallResult reflects semantic outcome

---

## Phase 5: User Story 3 - Admin Elevation (Priority: P3)

**Goal**: Detect and trigger admin elevation. Detect sudo on PATH, fall back to ShellExecuteExW runas.

**Independent Test**: Verify `is_elevated()` check and elevation path selection (sudo vs runas).

- [ ] T023 [P] [US3] Implement `is_elevated() -> bool` via `IsUserAnAdmin` in `crates/astro-up-core/src/install/elevation.rs` (cfg(windows))
- [ ] T024 [P] [US3] Implement `detect_sudo() -> bool` checking `sudo.exe` on PATH in `crates/astro-up-core/src/install/elevation.rs` (cfg(windows))
- [ ] T025 [US3] Implement `elevate_and_reexec(args: &[String])` in `crates/astro-up-core/src/install/elevation.rs` with sudo path via `tokio::process::Command` and runas path via `ShellExecuteExW`
- [ ] T026 [US3] Wire reactive elevation retry on exit code 740 into `InstallerService::install()` in `crates/astro-up-core/src/install/mod.rs`
- [ ] T027 [US3] Wire proactive elevation check (manifest `elevation = "required"`) before process spawn in `crates/astro-up-core/src/install/mod.rs`

**Checkpoint**: Elevation detected and triggered via sudo or runas, reactive retry on 740

---

## Phase 6: User Story 4 - Reboot Handling (Priority: P4)

**Goal**: Never auto-reboot. Return SuccessRebootRequired and emit event for reboot-required exit codes.

**Independent Test**: Verify exit code 3010 returns SuccessRebootRequired and emits InstallRebootRequired event.

- [ ] T028 [US4] Wire reboot handling into exit code interpretation to return `InstallResult::SuccessRebootRequired` and emit `InstallRebootRequired` event in `crates/astro-up-core/src/install/mod.rs`

**Checkpoint**: Reboot-required is a success state, event emitted, never auto-reboots

---

## Phase 7: User Story 5 - Uninstall (Priority: P5)

**Goal**: Uninstall packages via registry uninstall string or directory deletion. Support `upgrade_behavior = "uninstall_previous"`.

**Independent Test**: Verify registry lookup returns uninstall string, silent uninstall runs.

- [ ] T029 [P] [US5] Implement `find_uninstall_command(package_id: &str) -> Option<String>` reading registry `QuietUninstallString` and `UninstallString` in `crates/astro-up-core/src/install/uninstall.rs` (cfg(windows))
- [ ] T030 [P] [US5] Implement silent uninstall execution (append silent switches to uninstall command) in `crates/astro-up-core/src/install/uninstall.rs`
- [ ] T031 [US5] Implement ZIP/portable uninstall by deleting install directory in `crates/astro-up-core/src/install/uninstall.rs`
- [ ] T032 [US5] Implement `InstallerService::uninstall()` in `crates/astro-up-core/src/install/mod.rs`
- [ ] T033 [US5] Wire `upgrade_behavior = "uninstall_previous"` flow in `InstallerService::install()` to uninstall current before installing new in `crates/astro-up-core/src/install/mod.rs`

**Checkpoint**: Uninstall works for registry-based and ZIP/portable packages, upgrade_behavior wired

---

## Phase 8: User Story 6 - Install Directory Override (Priority: P6)

**Goal**: Pass custom install directory via type-appropriate switches.

**Independent Test**: Verify correct switch generated for InnoSetup (/DIR=), MSI (INSTALLDIR=), NSIS (/D=).

- [ ] T034 [P] [US6] Add install directory switch patterns per InstallMethod to `crates/astro-up-core/src/install/switches.rs` for InnoSetup, MSI, NSIS, WiX
- [ ] T035 [P] [US6] Wire install directory override into switch resolution in `crates/astro-up-core/src/install/switches.rs` to append directory switch when `install_dir` is Some
- [ ] T036 [US6] Add unit tests for directory override switches in `crates/astro-up-core/tests/install_switches.rs`

**Checkpoint**: Custom install directory passed to all applicable installer types

---

## Phase 9: Polish and Cross-Cutting Concerns

**Purpose**: ZIP extraction, hooks, ledger, Job Objects, DownloadOnly, metrics

### ZIP Extraction (FR-008, FR-009)

- [ ] T037 [P] Implement `extract_zip()` in `crates/astro-up-core/src/install/zip.rs` using `enclosed_name()` for zip-slip protection, reject entries returning None
- [ ] T038 [P] Implement single-root-directory detection and flattening in `crates/astro-up-core/src/install/zip.rs` by scanning all entry prefixes and stripping common root
- [ ] T039 Write ZIP extraction tests in `crates/astro-up-core/tests/install_zip.rs` covering normal archive, zip-slip attack, single-root, multi-root, empty archive, files-only

### Process Tree Waiting (FR-015)

- [ ] T040 Implement `spawn_with_job_object()` in `crates/astro-up-core/src/install/process.rs` (cfg(windows)) using `CreateProcessW` with `CREATE_SUSPENDED`, Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, assign process, resume thread, wait via `spawn_blocking`
- [ ] T041 Wire Job Object path for bootstrapper installer types (Burn) in `crates/astro-up-core/src/install/mod.rs`

### Hooks (FR-011)

- [ ] T042 Implement `run_hook(command: &str, elevated: bool)` in `crates/astro-up-core/src/install/hooks.rs` detecting .ps1 for PowerShell else cmd /c with 60s timeout
- [ ] T043 Wire pre_install (abort on failure) and post_install (warn on failure) hooks in `crates/astro-up-core/src/install/mod.rs`

### Ledger Recording (FR-020)

- [ ] T044 Implement `record_install()` in `crates/astro-up-core/src/install/ledger.rs` writing LedgerEntry with package_id, version, install_path, source=AstroUp
- [ ] T045 Wire ledger recording after successful install in `crates/astro-up-core/src/install/mod.rs`

### DownloadOnly and Cancellation (FR-013, FR-014)

- [ ] T046 Implement DownloadOnly handling in `crates/astro-up-core/src/install/mod.rs` opening containing folder on Windows, returning Success on other platforms
- [ ] T047 Wire cancellation via CancellationToken in all spawn paths in `crates/astro-up-core/src/install/process.rs`

### Metrics and Final Polish

- [ ] T048 [P] Record `INSTALL_DURATION_SECONDS` metric in `crates/astro-up-core/src/install/mod.rs` after each install
- [ ] T049 [P] Add tracing spans and events for install operations in `crates/astro-up-core/src/install/mod.rs`
- [ ] T050 Run `cargo fmt` and `cargo clippy -- -D warnings` across workspace, fix any issues
- [ ] T051 Run `just check` to verify full CI passes, update any broken snapshot tests

---

## Dependencies and Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies, start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1, BLOCKS all user stories
- **Phases 3-8 (User Stories)**: All depend on Phase 2 completion
  - US1 (silent install) then US2 (exit codes) then US3 (elevation) are sequential
  - US4 (reboot) depends on US2 (exit code interpretation)
  - US5 (uninstall) can start after Phase 2 (independent of US1-US4)
  - US6 (directory override) can start after US1 (extends switches)
- **Phase 9 (Polish)**: ZIP (T037-T039) can start after Phase 2. Others depend on US1 orchestration.

### User Story Dependencies

```
Phase 2 (Foundational)
  |-- US1 (Silent Install) -- US2 (Exit Codes) -- US3 (Elevation)
  |                           |-- US4 (Reboot)
  |-- US5 (Uninstall) [independent]
  |-- US6 (Directory Override) [after US1 switches]
  |-- ZIP/Hooks/Ledger [after US1 orchestration]
```

### Parallel Opportunities

- T007, T008, T009, T011: foundational types (different files)
- T013, T014, T015: switch table, resolution, tests
- T020, T021: exit code implementation and tests
- T023, T024: elevation check and sudo detection
- T029, T030: registry lookup and uninstall execution
- T034, T035: directory switches and resolution
- T037, T038: ZIP extraction and single-root detection
- T048, T049: metrics and tracing

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T005)
2. Complete Phase 2: Foundational (T006-T012)
3. Complete Phase 3: US1 Silent Installation (T013-T019)
4. STOP and VALIDATE: Switch resolution works for all 10 types

### Incremental Delivery

1. Setup + Foundational: types compile, module wired
2. US1: silent install works
3. US2: exit codes interpreted
4. US3: elevation triggers
5. US4: reboot handled
6. US5: uninstall works
7. US6: directory override works
8. Edge Cases: ZIP, hooks, ledger, Job Objects, metrics
9. Polish: fmt, clippy, CI green

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- All cross-platform code must compile and test on Ubuntu CI
- All Windows-only code must be gated with cfg(windows)
- Commit after each completed task or logical group
