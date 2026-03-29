# Checklist: Repository Scaffolding — DX, CI/CD, Cross-Platform

**Purpose**: Validate requirement quality for developer experience, CI/CD, and cross-platform aspects
**Created**: 2026-03-29
**Feature**: [spec.md](../spec.md)
**Depth**: Standard
**Audience**: Reviewer (PR)

## Requirement Completeness

- [ ] CHK001 - Are minimum version requirements specified for all prerequisite tools (Rust, Node.js, pnpm)? [Completeness, Spec §Assumptions]
- [ ] CHK002 - Are Tauri v2 system dependencies (OS-specific build tools) enumerated per platform? [Completeness, Gap]
- [ ] CHK003 - Are all Justfile recipes defined with their expected behavior and exit codes? [Completeness, Spec §FR-015]
- [ ] CHK004 - Is the `just setup` recipe's scope fully specified — what does it install vs. what must be pre-installed? [Completeness, Spec §US1]
- [ ] CHK005 - Are frontend devDependency version constraints specified (exact vs. range)? [Completeness, Spec §FR-006]
- [ ] CHK006 - Are Tauri v2 capability permissions requirements listed (which permissions, why)? [Completeness, Gap]

## Requirement Clarity

- [ ] CHK007 - Is "placeholder page" in FR-007 defined with specific content criteria (what elements, what text)? [Clarity, Spec §FR-007]
- [ ] CHK008 - Is "default dimensions" in FR-005 quantified with specific width/height values? [Clarity, Spec §FR-005]
- [ ] CHK009 - Is "project conventions" in FR-010 (CLAUDE.md) scoped — which conventions must be documented? [Clarity, Spec §FR-010]
- [ ] CHK010 - Is "smoke test" in FR-017 defined — what constitutes a passing smoke test for each crate? [Clarity, Spec §FR-017]
- [ ] CHK011 - Are "common development tasks" in FR-015 exhaustively listed or open-ended? [Clarity, Spec §FR-015]

## Requirement Consistency

- [ ] CHK012 - Are Justfile recipe names consistent between spec (FR-015), plan (Justfile recipes table), and tasks (T021, T024)? [Consistency]
- [ ] CHK013 - Does US2 acceptance scenario 1 list the same checks as FR-008 CI jobs? [Consistency, Spec §US2, §FR-008]
- [ ] CHK014 - Are Rust workspace dependency versions consistent between plan (Dependencies section) and tasks (T001, T010)? [Consistency]

## Acceptance Criteria Quality

- [ ] CHK015 - Is SC-005 "hot-reload within 2 seconds" measurable — from what event to what event? [Measurability, Spec §SC-005]
- [ ] CHK016 - Is SC-006 "under 60 seconds" measured from cold start or warm cache? [Measurability, Spec §SC-006]
- [ ] CHK017 - Is SC-001 "under 5 minutes" measured from `git clone` or from `just setup`? [Measurability, Spec §SC-001]
- [ ] CHK018 - Can "all CI checks pass with zero manual intervention" (SC-003) be objectively verified on the first commit? [Measurability, Spec §SC-003]

## Scenario Coverage — CI/CD

- [ ] CHK019 - Are CI failure notification requirements specified (who gets notified, how)? [Coverage, Gap]
- [ ] CHK020 - Are CI caching requirements defined (Cargo registry, target, pnpm store)? [Coverage, Gap]
- [ ] CHK021 - Is the CI behavior specified when the Windows path filter matches but no cfg(windows) code exists yet? [Coverage, Spec §FR-008]
- [ ] CHK022 - Are Dependabot PR auto-merge or manual review requirements specified? [Coverage, Spec §FR-011]
- [ ] CHK023 - Are release-please version bump rules specified (major/minor/patch triggers)? [Coverage, Spec §FR-012]

## Scenario Coverage — Cross-Platform

- [ ] CHK024 - Are requirements specified for what happens when a developer lacks Tauri system dependencies? [Coverage, Spec §Edge Cases]
- [ ] CHK025 - Are macOS-specific build requirements documented (Xcode CLI tools version)? [Coverage, Gap]
- [ ] CHK026 - Is the behavior specified when `cargo tauri dev` runs on Linux without WebKit2GTK? [Coverage, Gap]
- [ ] CHK027 - Are requirements defined for which workspace features/crates are excluded on non-Windows? [Coverage, Spec §FR-018]

## Edge Case Coverage

- [ ] CHK028 - Is the behavior specified when `pnpm install` fails during `just setup`? [Edge Case, Gap]
- [ ] CHK029 - Is the behavior specified when `frontend/dist/` doesn't exist at Tauri build time? [Edge Case, Gap]
- [ ] CHK030 - Are requirements defined for concurrent `just dev` invocations (port conflict)? [Edge Case, Gap]

## Dependencies & Assumptions

- [ ] CHK031 - Is the assumption "Node.js v22+" validated — does PrimeVue 4 / Vite 6 require this minimum? [Assumption, Spec §Assumptions]
- [ ] CHK032 - Is the assumption "pnpm pre-installed" justified — why not npm or yarn? [Assumption, Spec §Assumptions]
- [ ] CHK033 - Is the Tauri v2 minimum version specified or assumed to be "latest"? [Assumption, Gap]
