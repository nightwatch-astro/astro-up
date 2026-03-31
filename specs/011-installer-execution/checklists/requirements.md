# Requirements Quality Checklist: Installer Execution

**Purpose**: Validate completeness, clarity, consistency, and measurability of spec 011 requirements before planning
**Created**: 2026-03-30
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Are silent switch defaults specified for all 10 installer types listed in FR-013? [Completeness, Spec §FR-001/§FR-013]
- [ ] CHK002 - Are `install_location` switch patterns documented for all applicable installer types (not just InnoSetup and MSI)? [Completeness, Spec §FR-007]
- [x] CHK003 - Is the `timeout` field for `InstallConfig` specified with type, default value, and validation constraints? [Gap, Spec §Edge Cases] → Resolved: FR-010 now specifies `Option<Duration>`, default 600s, range 10s–3600s, covers full process tree.
- [ ] CHK004 - Are requirements defined for what happens when `DownloadOnly` packages reach the install flow? [Completeness, Spec §FR-013/§Edge Cases]
- [ ] CHK005 - Are pre/post hook execution requirements (shell detection, timeout, elevation inheritance) captured as FRs or only in decisions.md? [Completeness, Spec §FR-011]
- [ ] CHK006 - Is the `InstallRequest` entity fully specified with all fields (package info, installer path, install dir, quiet, elevation, timeout, cancel token)? [Completeness, Spec §Key Entities]

## Requirement Clarity

- [x] CHK007 - Is "custom switches fully replace defaults" (FR-002) clear about what happens when manifest provides empty switches — does empty mean "no switches" or "use defaults"? [Clarity, Spec §FR-002] → Resolved: FR-002 now specifies empty `switches.silent = []` means "no switches"; missing section means "use defaults".
- [ ] CHK008 - Is "support admin elevation" (FR-005) specific about whether reactive elevation (exit code 740) applies to all installer types or only exe-based? [Clarity, Spec §FR-005]
- [ ] CHK009 - Is "wait for entire process tree" (FR-015) clear about the timeout boundary — does the 10-minute timeout cover the parent only or the full tree? [Clarity, Spec §FR-015/§FR-010]
- [ ] CHK010 - Is "with confirmation" for ZIP/portable uninstall (FR-017) defined — who confirms, how, and what happens on decline? [Clarity, Spec §FR-017]
- [ ] CHK011 - Is the `Cancelled` variant in `InstallResult` clear about when it's returned vs `CoreError::Cancelled`? [Clarity, Spec §Key Entities]

## Requirement Consistency

- [ ] CHK012 - Are the `InstallResult` enum variants consistent between Key Entities (Success, SuccessRebootRequired, Cancelled) and the exit code mapping in US2 (which also references Timeout)? [Consistency, Spec §Key Entities/§US2]
- [ ] CHK013 - Is the elevation behavior consistent between US3 (proactive from manifest + reactive from 740) and FR-005 (same)? [Consistency, Spec §US3/§FR-005]
- [ ] CHK014 - Are the `KnownExitCode` variants in Key Entities consistent with the error variants in `CoreError` (spec 003)? [Consistency, Cross-spec]
- [ ] CHK015 - Is the uninstall scope consistent between US5 ("packages with registered uninstaller") and FR-016/FR-017 (registry string vs directory deletion)? [Consistency, Spec §US5/§FR-016/§FR-017]

## Acceptance Criteria Quality

- [ ] CHK016 - Can SC-001 ("silent installation works for all supported installer types") be objectively measured — is there a definition of "works"? [Measurability, Spec §SC-001]
- [ ] CHK017 - Is SC-005 ("uninstall succeeds for all packages with registered uninstall strings") measurable given that not all packages have uninstall strings? [Measurability, Spec §SC-005]
- [ ] CHK018 - Are acceptance scenarios for US1 missing a ZIP with malicious paths case (covered in edge cases but not in US1 scenarios)? [Coverage, Spec §US1]
- [ ] CHK019 - Are acceptance scenarios for US3 missing the case where elevation is `prohibited` but installer needs admin? [Coverage, Spec §US3]

## Scenario Coverage

- [ ] CHK020 - Are requirements defined for concurrent install attempts (two packages installing simultaneously)? [Gap]
- [ ] CHK021 - Are requirements defined for what happens when a pre-install hook modifies the installer file or target directory? [Gap, Spec §FR-011]
- [ ] CHK022 - Are requirements defined for `upgrade_behavior = "deny"` — what error is returned and how is the user informed? [Gap, Spec §FR-018]
- [ ] CHK023 - Are requirements defined for installing when the target directory already exists and contains files (ZIP/Portable)? [Gap]
- [ ] CHK024 - Are requirements defined for what happens when the ledger write fails after a successful install? [Gap, Spec §FR-020]

## Edge Case Coverage

- [x] CHK025 - Is the behavior specified when an installer's exit code is in both `success_codes` and `known_exit_codes`? [Edge Case, Spec §FR-003/§InstallConfig] → Resolved: FR-003 now specifies `success_codes` takes priority; semantic meaning is informational only.
- [ ] CHK026 - Is the behavior specified for zero-byte installers or corrupt/truncated files? [Edge Case, Gap]
- [ ] CHK027 - Is the behavior specified when `ShellExecuteW runas` is denied by the user (UAC cancelled)? [Edge Case, Spec §FR-005]
- [ ] CHK028 - Is the ZIP single-root detection behavior specified for archives with only files (no directories) at the root? [Edge Case, Spec §FR-009]
- [ ] CHK029 - Is the behavior specified when `sudo.exe` exists on PATH but fails (e.g., sudo disabled in Windows settings)? [Edge Case, Spec §Clarifications]

## Cross-Spec Dependencies

- [ ] CHK030 - Is the `LedgerEntry.install_path` cross-spec change to spec 003 documented with migration/compatibility requirements? [Dependency, Spec §FR-020]
- [ ] CHK031 - Are the new `Event` variants (`InstallFailed`, `InstallRebootRequired`) documented with their serialization format for TypeScript consumers (spec 017)? [Dependency, Spec §FR-012]
- [x] CHK032 - Is the `Installer` trait signature change (`Result<InstallResult, CoreError>`) documented as a breaking change to spec 003's trait definitions? [Dependency, Cross-spec] → Resolved: Assumptions section now documents all 3 cross-spec changes (trait signature, LedgerEntry field, InstallConfig field) with compatibility notes.
- [ ] CHK033 - Are the config system dependencies (timeout default, install paths) specified with reference to spec 004 config keys? [Dependency, Spec §Assumptions]

## Non-Functional Requirements

- [ ] CHK034 - Are logging/tracing requirements specified for installer execution (what events are traced, at what level)? [Gap]
- [ ] CHK035 - Are metrics requirements beyond `INSTALL_DURATION_SECONDS` specified (e.g., success/failure counts, elevation frequency)? [Gap]
- [x] CHK036 - Are security requirements for hook execution specified — can hooks execute arbitrary commands, and is that an accepted risk? [Gap, Spec §FR-011] → Resolved: FR-011 now documents hooks as trusted (from signed manifests), accepted risk equivalent to running the installer itself.

## Notes

- Check items off as completed: `[x]`
- Items referencing `[Gap]` indicate missing requirements that should be added to the spec
- Items referencing `[Cross-spec]` require coordination with other spec owners
- Items are numbered sequentially (CHK001–CHK036) for easy reference
