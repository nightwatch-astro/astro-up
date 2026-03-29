# Retrospective: 003-core-domain-types

**Date**: 2026-03-29
**Outcome**: Complete — all 52 issues closed, 63 tests pass, zero clippy warnings
**Tasks**: 43 tasks implemented

## Findings

### 1. BackupManager trait was missed during implementation

**Category**: Spec Quality
**Severity**: High — phantom completion detected by verify-tasks
**What happened**: T031 specified 5 traits (Detector, Provider, Installer, Downloader, BackupManager). Implementation only created 4. The issue was closed without BackupManager being written.
**Root cause**: Implementing multiple traits in one file, easy to miss one. No test specifically checked for BackupManager's existence.
**Operationalization**: Each trait should have its own compilation test (e.g., `fn _assert_backup_manager_exists() { let _: fn() -> Box<dyn BackupManagerDyn> = ...; }`). Or: one issue per trait, not one issue for all 5 traits.

### 2. Enum validation tests (T026-T028) were never written

**Category**: Process
**Severity**: Medium — 3 test tasks skipped, detected by verify-tasks
**What happened**: Phase 4 (US2) issues were closed but the parameterized rstest tests were never implemented.
**Root cause**: Rushed from foundational phase to error/traits/events, skipping the test phase. No gating between phases.
**Operationalization**: Already addressed by one-issue-per-task pattern. Each task issue must have code evidence before closing.

### 3. Inline tests should be integration tests for public API

**Category**: Spec Quality
**Severity**: Low — refactored during cleanup
**What happened**: Error Display tests were inline `#[cfg(test)]` in `src/error.rs` using `super::*`. These test the public API and belong in `tests/`.
**Root cause**: Default habit of putting tests next to the code. For library crates, integration tests in `tests/` are more appropriate for public API validation.
**Operationalization**: Convention: unit tests inline for private logic, integration tests in `tests/` for public API behavior.

### 4. `provider: String` should be `provider: CheckMethod`

**Category**: Spec Quality
**Severity**: Low — caught during review
**What happened**: `CoreError::ProviderUnavailable` used a free-form `String` for the provider name, allowing typos.
**Root cause**: Spec said `provider` without specifying the type. Implementation defaulted to String.
**Operationalization**: Spec should specify enum types for all fields that reference a closed set of values.

### 5. TOML key types differ from Rust types

**Category**: Wiring
**Severity**: Low — spec updated
**What happened**: `known_exit_codes` was `HashMap<i32, KnownExitCode>` in spec but TOML keys must be strings. Implementation used `HashMap<String, KnownExitCode>`.
**Root cause**: Spec was written with Rust types in mind, not TOML serialization constraints.
**Operationalization**: When specifying HashMap key types, consider the serialization format. TOML keys are always strings.

### 6. Consistent snake_case naming requires manifest migration

**Category**: Process
**Severity**: Low — noted for Spec 002
**What happened**: Manifests use `innosetup` but types use `inno_setup` (proper snake_case). Tests use the new format.
**Root cause**: Historical Go naming was inconsistent. Rust serde derives enforce snake_case.
**Operationalization**: Manifest migration (Spec 002) must rename `innosetup` → `inno_setup`, `zipwrap` → `zip_wrap` in all 95 TOML files.

## Metrics

| Metric | Value |
|--------|-------|
| Issues created | 52 (9 parents + 43 tasks) |
| Issues closed | 52 |
| Tests | 63 (18 lib + 39 enum + 1 error display + 5 software serde) |
| Verify-tasks findings | 1 phantom (BackupManager), 3 partial (enum tests) |
| Verify findings | 1 diverged (HashMap key type), 7 partial (spec wording) |
| Lines of Rust | ~830 (types, errors, traits, events, ledger, release, metrics) |
