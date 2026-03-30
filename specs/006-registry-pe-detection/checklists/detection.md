# Detection Requirements Quality Checklist: Software and Driver Detection

**Purpose**: Validate requirement completeness, clarity, and cross-spec consistency for the detection system
**Created**: 2026-03-30
**Feature**: [spec.md](../spec.md)
**Focus**: Cross-spec dependencies (003/004/005) + functional + non-functional requirements

## Cross-Spec Dependency Completeness

- [ ] CHK001 - Is the interface between detection and the catalog (spec 005) specified — how does detection obtain the full package list for scanning? [Gap, Spec §FR-001]
- [ ] CHK002 - Does the spec define which fields from `Software` (spec 003) are consumed by each detection method — registry_key, file_path, etc.? [Completeness, Spec §FR-005]
- [ ] CHK003 - Is the dependency on path token expansion (deferred from spec 004) acknowledged with a concrete resolution plan, or is it left as an implicit assumption? [Gap, Spec §Assumptions]
- [ ] CHK004 - Are the `DetectionConfig.fallback` chain semantics defined — does detection stop at first success, or continue to validate across methods? [Clarity, Spec §FR-004]
- [ ] CHK005 - Is the relationship between `LedgerEntry.version` and detection-returned version specified — what happens when they disagree? [Gap, Spec §FR-007/FR-008]
- [ ] CHK006 - Does the spec define what "auto-managed" means in terms of persistence — is a new entity created, or is the detection result itself the managed state? [Clarity, Spec §Clarifications/FR-019]

## Detection Method Consistency

- [ ] CHK007 - Are the input requirements (which manifest fields are needed) consistently defined for each detection method: registry, PE, WMI, ASCOM, file_exists, config_file? [Consistency, Spec §FR-001 through FR-013]
- [ ] CHK008 - Is the output contract (DetectionResult enum) consistently applied across all detection methods, including the "reason" field for Unavailable? [Consistency, Spec §Key Entities]
- [ ] CHK009 - Are the error-to-DetectionResult mappings specified for each method — e.g., permission denied → Unavailable vs NotInstalled? [Clarity, Spec §FR-010]
- [ ] CHK010 - Is the `config_file` detection method (FR-011) specified with the same level of detail as registry and PE — what file format, what version extraction logic? [Completeness, Spec §FR-011]
- [ ] CHK011 - Are the `file_exists` method requirements clear — does it return InstalledUnknownVersion or does it attempt version extraction? [Clarity, Spec §FR-011]

## Requirement Clarity

- [ ] CHK012 - Is "configurable registry values (default: DisplayVersion)" (FR-002) specified with the full list of supported values, or is DisplayVersion the only one with a fallback? [Clarity, Spec §FR-002]
- [ ] CHK013 - Is the WMI filter combination (DriverProviderName + DeviceClass + InfName) specified as AND or OR logic? [Ambiguity, Spec §FR-013]
- [ ] CHK014 - Is "wildcard VID:PID matching" (FR-015) specified with exact syntax — does `03C3:*` mean glob-style or regex? Are other patterns like `*:120A` supported? [Clarity, Spec §FR-015]
- [ ] CHK015 - Is the version string parsing boundary defined — which raw strings are valid input to Version::parse, and what happens with unparseable versions (e.g., "beta", "2024.1.2.3.4")? [Clarity, Spec §FR-008]

## Non-Functional Requirements

- [ ] CHK016 - Is the 5-second scan target (SC-001) defined under specific conditions — cold vs warm cache, HDD vs SSD, number of packages? [Measurability, Spec §SC-001]
- [ ] CHK017 - Is the WMI 10-second timeout (FR-018) specified per-query or total across all WMI detection methods in a scan? [Clarity, Spec §FR-018]
- [ ] CHK018 - Are platform behavior requirements explicit for non-Windows — does each method define its non-Windows return value, or only the blanket "Unavailable" assumption? [Completeness, Spec §FR-017/Assumptions]

## Edge Case & Scenario Coverage

- [ ] CHK019 - Are requirements defined for packages that appear in the catalog but have NO detection config at all — skip silently or report as "unknown"? [Gap]
- [ ] CHK020 - Is the behavior specified when a package is detected by multiple methods in the chain with different versions — which version wins? [Gap, Spec §FR-004]
- [ ] CHK021 - Are requirements defined for concurrent scan requests — can two scans run in parallel, or is there mutual exclusion? [Gap]
- [ ] CHK022 - Is the cache invalidation behavior for "auto-managed" packages specified — if a package was managed but is no longer detected (uninstalled externally), what happens on next scan? [Gap, Spec §FR-016/FR-019]
- [ ] CHK023 - Are VID:PID discovery results specified in relation to scan results — does discovery run as part of a scan, or is it a separate operation? [Gap, Spec §User Story 4]

## Notes

- Cross-spec exploration found that **path token expansion was dropped from spec 004** — spec 006 depends on it but no resolution plan is documented (CHK003)
- `PackageId` newtype exists in spec but implementation status is in-progress (PR #167 aligned Software.id)
- Catalog `CatalogReader::list_all()` is the likely entry point for full-catalog scanning (CHK001)
- `DetectionConfig.fallback` supports recursive chaining via `Option<Box<DetectionConfig>>` — spec should clarify chain termination semantics (CHK004)
