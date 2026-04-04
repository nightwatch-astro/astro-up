# Lifecycle CI Checklist: Automated Package Lifecycle Testing & Detection Discovery

**Purpose**: Validate requirement completeness, clarity, and consistency for the lifecycle testing workflow, detection discovery, and cross-repo schema changes
**Created**: 2026-04-04
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 - Are requirements defined for how the workflow determines the "latest discovered version" when no explicit version input is provided? [Completeness, Spec §FR-020] — Resolved: reads from `versions/{id}/` directory in manifests repo
- [x] CHK002 - Are requirements defined for what happens when a package has `[detection]` already and the workflow discovers a different config? [Gap] — Resolved: compare and only PR if different (FR-017)
- [x] CHK003 - Is the format of the auto-created PR body specified? [Completeness, Spec §FR-017] — Resolved: package info, phase summary table, TOML code fence, workflow run link
- [x] CHK004 - Are requirements defined for how the matrix sweep identifies packages missing detection? [Completeness, Spec §FR-014] — Resolved: scan `manifests/*.toml` for presence of `[install]` but absence of `[detection]`
- [x] CHK005 - Is the CLI subcommand interface specified — command name, arguments, output format, exit codes? [Gap] — Resolved: FR-030 through FR-033 added
- [x] CHK006 - Are requirements specified for how the workflow clones/accesses the manifests repo? [Gap, Spec §FR-019] — Resolved: `actions/checkout` with Nightwatch app token

## Requirement Clarity

- [x] CHK007 - Is "case-insensitive substring matching" for registry discovery sufficiently precise? [Clarity, Spec §FR-029] — Clear: manifest `name` as primary match, package ID as fallback. Short IDs not a concern (shortest is 7 chars)
- [x] CHK008 - Is "recursive fallback chain" for the expanded schema clearly bounded? [Clarity, Spec §FR-021] — Resolved: capped at 3 levels
- [x] CHK009 - Is the confidence ranking in FR-002 an ordered list or a scoring system? [Clarity, Spec §FR-002] — Resolved: ordered priority list with tie-breaking rules
- [x] CHK010 - Is "best-effort uninstall" in the cleanup step defined? [Clarity, Spec §FR-016] — Resolved: 3-tier (uninstall → kill processes → remove dir with user confirmation)

## Requirement Consistency

- [x] CHK011 - Are the 7 detection methods listed in FR-001 consistent with the DetectionMethod enum? [Consistency, Spec §FR-001] — Verified: exact match
- [x] CHK012 - Does FR-024's install path fallback chain align with User Story 4? [Consistency, Spec §FR-024] — Verified: consistent
- [x] CHK013 - Are the schema fields consistent between FR-021, FR-022, and FR-027? [Consistency] — Verified: same list in all three

## Acceptance Criteria Quality

- [x] CHK014 - Can SC-002's "80% accuracy" be objectively measured? [Measurability, Spec §SC-002] — Deferred to planning: compare discovered vs hand-written configs
- [x] CHK015 - Is SC-004's "zero dirty runner incidents" measurable? [Measurability, Spec §SC-004] — Clear: runners are ephemeral, each package is its own job
- [x] CHK016 - Is SC-006's "valid TOML" testable? [Measurability, Spec §SC-006] — Deferred to planning: TOML parse validation before PR creation

## Scenario Coverage

- [x] CHK017 - Are requirements defined for autoupdate URLs requiring auth or CDN redirects? [Coverage, Gap] — N/A: all downloads are public, reqwest handles redirects
- [x] CHK018 - Are requirements defined for dry-run when file is neither PE nor ZIP? [Coverage, Spec §FR-026] — Clear: falls back to FileExists, no version
- [x] CHK019 - Are requirements specified for `download_only` packages? [Coverage, Spec §FR-008] — Resolved: skip install/uninstall, require `--install-dir` flag
- [x] CHK020 - Are requirements defined when PR branch already exists? [Coverage, Spec §FR-018] — Resolved: force-push and update existing PR

## Edge Case Coverage

- [x] CHK021 - Is behavior defined for multiple matching registry entries? [Edge Case, Spec §FR-029] — Clear: HKLM 64→32→HKCU priority, prefer entry with DisplayVersion
- [x] CHK022 - Is behavior specified when registry uninstall command differs from install method? [Edge Case, Spec §FR-010] — Clear: always use registry-discovered command
- [x] CHK023 - Are requirements defined for packages installing services/drivers? [Edge Case, Spec §FR-016] — Clear: cleanup reports failure, ephemeral runners handle the rest
- [x] CHK024 - Is behavior specified when installer changes PATH? [Edge Case] — Clear: detection uses absolute paths, not PATH

## Cross-Repo Requirements

- [x] CHK025 - Is the schema version numbering defined? [Completeness, Spec §FR-028] — Resolved: stays at v1, app not published
- [x] CHK026 - Are deployment ordering requirements defined? [Gap, Spec §FR-028] — N/A: no backward compat needed, single schema version
- [x] CHK027 - Is the fallback serialization format specified? [Clarity, Spec §FR-021] — Resolved: JSON blob in `fallback_config` column (FR-034)
- [x] CHK028 - Are requirements defined for unknown/future detection fields? [Coverage, Spec §FR-027] — Clear: serde ignores unknown fields by default

## Dependencies & Assumptions

- [x] CHK029 - Is the "Windows runners run as Admin" assumption validated? [Assumption] — Clear: documented by GitHub, well-known
- [x] CHK030 - Is the "Software struct deserializes TOML" assumption validated? [Assumption] — Clear: confirmed serde derives + `#[serde(default)]` on all optional fields
- [x] CHK031 - Are Nightwatch GitHub App token permissions documented? [Dependency] — Clear: proven by release.yml cross-repo operations

## Notes

- All 31 items reviewed and resolved
- 2 items deferred to planning (CHK014 validation procedure, CHK016 TOML validation mechanism)
- 2 deferred issues created: #690 (file browser picker), #691 (install --install-dir flag)
- Spec updated with 7 new/modified FRs from checklist findings
