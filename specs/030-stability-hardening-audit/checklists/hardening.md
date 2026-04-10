# Hardening Audit Checklist: Stability and Hardening Audit

**Purpose**: Validate requirements completeness, clarity, and consistency for the 7-phase codebase hardening audit
**Created**: 2026-04-10
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Are all path-accepting Tauri commands enumerated in Phase A requirements? The audit found `clear_directory`, `create_backup`, `delete_backup`, and backup restore — are there additional commands not covered? [Completeness, Spec §FR-001–010]
- [x] CHK002 - Are requirements defined for handling ZIP archives with absolute paths (e.g., `/etc/passwd` as entry name), not just relative `..` traversal? [Completeness, Spec §FR-001] — RESOLVED: FR-001 expanded to reject absolute paths
- [ ] CHK003 - Is the list of "critical" vs "non-critical" spawned tasks explicitly defined? FR-013 mentions event forwarding and progress streaming — are these exhaustive or are additional spawn sites unclassified? [Completeness, Spec §FR-012–013]
- [ ] CHK004 - Are requirements specified for the 7 `#[allow(dead_code)]` fields found in `UpdateOrchestrator`? Phase D covers structure but doesn't explicitly address dead code disposition. [Gap]
- [ ] CHK005 - Does Phase G cover the `tokio "full"` finding for ALL crates or only `astro-up-cli`? The audit finding was specific to CLI but the requirement (FR-030) is generic. [Completeness, Spec §FR-030]
- [x] CHK006 - Are requirements defined for what happens when a download server omits the `Content-Length` header entirely? FR-009 assumes the size is known before download begins. [Gap, Spec §FR-009] — RESOLVED: FR-009 now covers streaming with running byte counter
- [ ] CHK007 - Are requirements specified for validating config file integrity (corruption), not just size? FR-010 covers size but not malformed content. [Gap, Spec §FR-010]

## Requirement Clarity

- [x] CHK008 - Is "application-controlled directories" in FR-004 defined with a concrete list or derivation rule? The allowlist source (hardcoded, config-derived, or runtime-computed) is unspecified. [Clarity, Spec §FR-004] — RESOLVED: FR-004 now specifies config-derived allowlist (backup dir, per-package config paths, app cache dir)
- [x] CHK009 - Is "aggregate size limits" in FR-005 quantified with a specific threshold? No numeric limit is stated for backup creation. [Clarity, Spec §FR-005] — RESOLVED: FR-005 now specifies 1 GB aggregate limit
- [ ] CHK010 - Is "compile-time constants" in FR-015 defined precisely? The boundary between an `unwrap()` on a compile-time constant (acceptable) and a runtime value (unacceptable) needs a clear rule. [Clarity, Spec §FR-015]
- [ ] CHK011 - Is "non-meaningful" in FR-018 (stderr writes exempted from logging) defined with clear criteria? Which discarded results qualify as non-meaningful? [Ambiguity, Spec §FR-018]
- [x] CHK012 - Is "focused sub-functions" in FR-020 quantified? The user story says "each under 50 lines" but the FR does not include this constraint. [Clarity, Spec §FR-020] — RESOLVED: FR-020 now includes "each under 50 lines"
- [ ] CHK013 - Is "domain-focused modules" in FR-022 defined with a decomposition strategy (by feature, by layer, by command type)? [Clarity, Spec §FR-022]
- [ ] CHK014 - Is "appropriate structured fields" in FR-023 defined with a minimum set of required fields per function category (I/O, network, database)? [Clarity, Spec §FR-023]

## Requirement Consistency

- [x] CHK015 - Do FR-019 (consolidate command handlers) and FR-022 (decompose >500-line files) align on the target structure for `commands.rs`? Consolidating handlers reduces duplication but decomposing by domain moves handlers to separate files — the spec should define the intended outcome. [Consistency, Spec §FR-019 vs §FR-022] — RESOLVED: FR-019 now specifies consolidate first, then decompose by domain
- [ ] CHK016 - Does the 500-line threshold in FR-022/SC-006 account for the test code exemption in Assumptions? The exemption says "primarily test code" — is 51% test code sufficient? [Consistency, Spec §FR-022 vs Assumptions]
- [ ] CHK017 - Are the Phase priorities (A=CRITICAL/HIGH, B=HIGH, etc.) consistent with the user story priorities (P1/P2/P3)? US5 (code structure) is P3 but Phase D is labeled HIGH/MEDIUM. [Consistency]

## Acceptance Criteria Quality

- [ ] CHK018 - Is SC-003 ("zero unwrap() calls") measurable without manual inspection? Does it define an automated verification method (e.g., clippy lint rule)? [Measurability, Spec §SC-003]
- [ ] CHK019 - Is SC-005 ("100% of public async I/O functions have tracing") measurable? How is the set of "public async I/O functions" enumerated for verification? [Measurability, Spec §SC-005]
- [ ] CHK020 - Is SC-006 ("no source file exceeds 500 lines") scoped to include or exclude auto-generated files (e.g., Tauri bindings, build.rs output)? [Measurability, Spec §SC-006]
- [ ] CHK021 - Is SC-007 ("all error-discarding patterns preceded by log or documented as silent") verifiable by grep/lint, or does it require manual judgment on "intentionally silent"? [Measurability, Spec §SC-007]

## Scenario Coverage

- [ ] CHK022 - Are requirements defined for the user experience during "degraded mode" after critical task restart budget exhaustion? FR-013 says "alert via UI notification" but doesn't specify what functionality is reduced or how the user is informed of limitations. [Coverage, Spec §FR-013]
- [ ] CHK023 - Are requirements defined for the behavior of the mounted-flag pattern when a component remounts quickly (e.g., route oscillation)? FR-027 covers unmount but not rapid remount. [Coverage, Spec §FR-027]
- [ ] CHK024 - Are requirements defined for how `parking_lot::Mutex` interacts with Tauri's state management lifecycle (app shutdown, plugin teardown)? [Coverage, Spec §FR-011]

## Edge Case Coverage

- [ ] CHK025 - Are requirements defined for concurrent backup restore and delete operations on the same archive? Path validation applies to each independently, but interleaving is unspecified. [Edge Case, Gap]
- [x] CHK026 - Are requirements defined for symlink detection on Windows (where symlinks require elevated privileges)? FR-003/FR-005 mandate symlink rejection but Windows symlink behavior differs from Unix. [Edge Case, Spec §FR-003, §FR-005] — RESOLVED: FR-003 now also rejects Windows reparse points and junctions
- [x] CHK027 - Are requirements defined for the case where a file being restored already exists at the target path? Overwrite, skip, or error? [Edge Case, Spec §FR-002] — RESOLVED: FR-002 now specifies overwrite (full replacement, not merge)

## Non-Functional Requirements

- [ ] CHK028 - Are performance impact requirements specified for path validation? Adding validation to every file operation could introduce measurable latency — is this acceptable without bounds? [Gap]
- [ ] CHK029 - Are requirements specified for the error message content shown to users? FR-017 says "appropriate user feedback" — is there a UX guideline for error message tone, detail level, or actionability? [Gap, Spec §FR-017]

## Dependencies & Assumptions

- [ ] CHK030 - Is the assumption that `parking_lot::Mutex` is compatible with Tauri state validated against Tauri v2's actual state management implementation? [Assumption]
- [ ] CHK031 - Is the assumption that TypeScript 6 / vue-tsc 3 can be deferred validated? Could outdated type-checking mask errors introduced by Phase F frontend changes? [Assumption]
- [ ] CHK032 - Is the assumption that 2 GB max download size is sufficient documented with evidence (largest known installer in the catalog)? [Assumption, Spec §FR-009]

## Notes

- Check items off as completed: `[x]`
- Items referencing `[Gap]` indicate missing requirements that should be added to the spec
- Items referencing `[Ambiguity]` indicate unclear requirements that need sharpening
- Items referencing `[Consistency]` indicate potential conflicts between spec sections
