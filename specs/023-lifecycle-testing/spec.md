# Feature Specification: Automated Package Lifecycle Testing & Detection Discovery

**Feature Branch**: `023-lifecycle-testing`
**Created**: 2026-04-04
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Project ID**: PVT_kwDOECmZr84BTDgZ
**Input**: User description: "Automated package lifecycle testing and detection discovery workflow"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Single Package Lifecycle Test (Priority: P1)

A maintainer wants to discover the detection config for a package that has an install method defined in its manifest but no detection section yet. They trigger a workflow run specifying the package ID. The system downloads the installer, installs it on a clean Windows environment, probes the system for all possible detection signatures (registry entries, PE files, config files, etc.), ranks the results by reliability, uninstalls the package, verifies clean removal, and outputs a complete `[detection]` TOML config. The maintainer reviews the output and merges the auto-created PR into the manifests repository.

**Why this priority**: This is the core value proposition — going from "no detection config" to "working detection config" for a single package, fully automated.

**Independent Test**: Trigger the workflow for a well-known package (e.g., NINA, PHD2), verify that the output detection config correctly identifies the installed version and path, and that uninstall leaves the system clean.

**Acceptance Scenarios**:

1. **Given** a package with an `[install]` section but no `[detection]` section in its manifest, **When** the maintainer triggers the lifecycle test workflow with that package ID, **Then** the system downloads, installs, discovers detection signatures, uninstalls, and outputs a valid `[detection]` TOML config.
2. **Given** a successful lifecycle test run, **When** the detection config is output, **Then** a PR is auto-created against the manifests repository containing the discovered `[detection]` section.
3. **Given** a package that fails to install, **When** the install step fails, **Then** the system reports the failure with logs, skips detection/uninstall, and does not create a PR.
4. **Given** a package that installs but leaves no discoverable signatures, **When** detection probing finds nothing, **Then** the system reports "detection unknown" with all probed locations for manual review.

---

### User Story 2 - Dry Run for Portable/Data Packages (Priority: P2)

A maintainer wants to discover detection info for a portable application, firmware file, or data resource that doesn't have a traditional installer. They trigger the workflow in dry-run mode. The system downloads the file, probes for file existence and version info (PE headers if applicable), but skips install/uninstall phases. The output indicates the appropriate detection method (e.g., FileExists) or marks the package as not applicable for detection.

**Why this priority**: ~20% of packages in the catalog are portable apps, firmware, or data files. These need detection support but cannot go through the full install/uninstall cycle.

**Independent Test**: Trigger dry-run for a known portable package (e.g., a standalone astronomy tool). Verify that it reports FileExists or not_applicable without attempting to run an installer.

**Acceptance Scenarios**:

1. **Given** a portable application package, **When** the maintainer triggers the workflow in dry-run mode, **Then** the system downloads the file, checks for PE version info, and outputs a FileExists detection config with the expected path.
2. **Given** a firmware blob or data file, **When** dry-run discovers no meaningful detection method, **Then** the system reports "not_applicable" with a reason (e.g., "firmware file — no installed state").
3. **Given** dry-run mode, **When** the workflow completes, **Then** no install or uninstall operations were attempted.

---

### User Story 3 - Matrix Sweep for Missing Detection (Priority: P2)

A maintainer wants to bulk-discover detection configs for all packages that are missing them. They trigger the matrix sweep manually via workflow_dispatch. The system identifies all packages with `[install]` but no `[detection]` section, runs the lifecycle test for each as parallel matrix jobs (max 5 concurrent), and creates individual PRs for each discovered config.

**Why this priority**: One-by-one discovery is correct but slow. Matrix sweep enables bootstrapping detection for the entire catalog.

**Independent Test**: Trigger matrix sweep on a subset of 3-5 packages missing detection. Verify that each package runs independently and produces its own PR.

**Acceptance Scenarios**:

1. **Given** 5 packages missing detection configs, **When** the matrix sweep runs, **Then** each package runs as an independent job with a 10-minute timeout.
2. **Given** a matrix sweep, **When** 3 packages succeed and 2 fail, **Then** 3 PRs are created for the successes and the 2 failures are reported with logs without failing the overall workflow.
3. **Given** concurrent matrix jobs, **When** multiple packages are tested, **Then** no shared state exists between jobs (each runs on a fresh runner).

---

### User Story 4 - Install Path Recording During Normal Installs (Priority: P3)

When a user installs a package through the GUI or CLI (not just in this workflow), the application records the discovered install path in the ledger. This path is later used by the backup system to resolve `{install_dir}` tokens in backup config paths.

**Why this priority**: The lifecycle workflow validates that install path recording works, but the actual user benefit is in the backup system. This is an enhancement to the existing install pipeline.

**Independent Test**: Install a package via CLI, verify the ledger contains the correct install path, then verify the backup system can resolve `{install_dir}` using that path.

**Acceptance Scenarios**:

1. **Given** a successful package install, **When** the install completes, **Then** the system runs detection to discover the install path and records it in the ledger.
2. **Given** a registry-detected package, **When** InstallLocation is present in the registry, **Then** the install path in the ledger matches the registry InstallLocation value.
3. **Given** a package where detection yields no install path, **When** recording the ledger entry, **Then** the system falls back to registry InstallLocation, then to the default install directory.

---

### User Story 5 - Install/Uninstall Regression Testing (Priority: P3)

A maintainer wants to verify that the install and uninstall pipelines work correctly for a package after making changes to the installer logic. They trigger the lifecycle test for specific packages. The phase-by-phase pass/fail report serves as a regression test — any failure indicates a regression in the install/uninstall pipeline.

**Why this priority**: Secondary benefit of the lifecycle workflow — it doubles as a regression test suite. However, the primary value is detection discovery.

**Independent Test**: Run lifecycle test for a package known to install/uninstall cleanly. Verify all phases pass. Intentionally break a switch and verify the failure is reported.

**Acceptance Scenarios**:

1. **Given** a package with a known-working install/uninstall cycle, **When** the lifecycle test runs, **Then** all phases (download, install, detect, verify-install, uninstall, verify-removal) pass.
2. **Given** a lifecycle test, **When** the uninstall phase fails, **Then** the report clearly indicates which phase failed, with the exit code and relevant logs.
3. **Given** a lifecycle test that fails during install, **When** the failure is reported, **Then** the system still attempts cleanup (best-effort uninstall or process termination).

---

### Edge Cases

- What happens when a package installer requires user interaction despite silent switches? The system times out after 10 minutes and reports the failure.
- What happens when an installer modifies system PATH or requires a reboot? The system treats reboot-required exit codes (3010, 1641) as success and continues with detection.
- What happens when the uninstaller leaves orphan files or registry keys? The verify-removal phase detects and reports them as warnings.
- What happens when two versions of the same package are installed side-by-side? Detection discovery reports all found entries; the output config targets the version that was just installed.
- What happens when the autoupdate URL is invalid or unreachable? The download phase fails, the system skips subsequent phases and reports the error.
- What happens when the GitHub Actions runner lacks sufficient disk space? The download phase checks available space before downloading and fails early if insufficient.
- What happens when a package has no `[checkver.autoupdate]` URL? The system reports that the package cannot be tested (no download source).
- What happens with `download_only` packages? The user must provide a target directory via `--install-dir`. The file is downloaded there, detection probes for file existence/PE headers, and install/uninstall phases are skipped. If `--install-dir` is not provided for a `download_only` package, the CLI errors with a clear message.

## Requirements *(mandatory)*

### Functional Requirements

**Detection Discovery**

- **FR-001**: System MUST probe all 7 detection methods (Registry, PeFile, FileExists, ConfigFile, AscomProfile, Wmi, DriverStore) when discovering detection signatures for a package.
- **FR-002**: System MUST rank discovery results by confidence using an ordered priority list: Registry with version and PeFile with version are high confidence, Registry without version and FileExists are medium, WMI/ASCOM/DriverStore are low. Tie-breaking: prefer the method that returns a version; if both return a version, prefer Registry (most stable across updates).
- **FR-003**: System MUST generate a complete detection config from the best discovery result, including fallback chain from second-best result.
- **FR-004**: System MUST use path tokens ({program_files}, {program_files_x86}, etc.) in generated configs rather than hardcoded absolute paths.
- **FR-005**: System MUST extract all available metadata from registry entries: DisplayVersion, InstallLocation, UninstallString, QuietUninstallString, Publisher, DisplayName.
- **FR-029**: Registry discovery MUST match primarily on the manifest `name` field (the human-readable product name) using case-insensitive substring matching against the registry DisplayName value, with the package ID as a fallback if the name yields no matches.

**Lifecycle Phases**

- **FR-006**: System MUST execute lifecycle phases in order: download, install, detect, verify-install, uninstall, verify-removal, report.
- **FR-007**: System MUST verify downloads by checking file existence and non-zero size. Hash verification MUST be attempted if the manifest provides a hash URL.
- **FR-008**: System MUST use the manifest's `[install]` method and silent switches for installation, reusing the existing installer pipeline. For `download_only` packages, the system MUST skip install and uninstall phases, download the file to a user-specified directory (via `--install-dir` flag, required for `download_only`), and probe for FileExists/PE detection only.
- **FR-009**: System MUST handle elevation requirements — CI Windows runners run as admin, so elevation checks should succeed by default.
- **FR-010**: System MUST discover the uninstall command from the registry (QuietUninstallString preferred over UninstallString) and execute it with silent switches.
- **FR-011**: System MUST re-run detection after uninstall to verify the package is no longer found. Any leftover files or registry keys MUST be reported as warnings.
- **FR-012**: System MUST produce a per-phase pass/fail report with timing, exit codes, and relevant log excerpts. Report MUST be rendered as a GitHub Actions job summary (markdown) and uploaded as a JSON artifact.

**Workflow Automation**

- **FR-013**: System MUST support single-package mode via workflow_dispatch with `package_id` (required), `version` (optional), and `dry_run` (boolean) inputs.
- **FR-014**: System MUST support matrix sweep mode (manual workflow_dispatch only, no scheduled runs) that identifies all packages missing detection by scanning `manifests/*.toml` files for the presence of `[install]` but absence of `[detection]` sections, and runs them as parallel jobs with a concurrency limit of 5.
- **FR-015**: System MUST enforce a 10-minute timeout per package lifecycle test.
- **FR-016**: System MUST clean up after itself using a cleanup step that always runs. Cleanup MUST: (1) attempt the registry-discovered uninstall command with silent switches, (2) if uninstall fails, kill any processes matching the installer filename, (3) only remove the install directory if the user explicitly confirms (via CLI prompt or workflow input). No reboot or service stop attempts.
- **FR-017**: System MUST auto-create a PR against the manifests repository with the discovered `[detection]` TOML section. If the package already has a `[detection]` section, the system MUST compare the discovered config with the existing one and only create a PR if they differ. The PR body MUST include: package name and version tested, phase-by-phase pass/fail summary table, the discovered `[detection]` TOML in a code fence, and a link to the workflow run.
- **FR-018**: System MUST be idempotent — running twice for the same package produces the same detection config. If the PR branch already exists from a previous run, the workflow MUST force-push to update it and update the existing PR.

**CLI Interface**

- **FR-030**: System MUST provide a `lifecycle-test` CLI subcommand with the following interface: `astro-up lifecycle-test <package_id> --manifest-path <path> [--version <ver>] [--install-dir <path>] [--dry-run] [--json]`. The `--install-dir` flag is required for `download_only` packages.
- **FR-031**: The `--manifest-path` argument MUST point to the root of the manifests repo checkout (containing `manifests/` and `versions/` directories).
- **FR-032**: The CLI MUST use structured exit codes: 0 = all phases passed, 1 = install failed, 2 = discovery failed (installed but no detection found), 3 = uninstall or verification failed, 4 = download failed.
- **FR-033**: With `--json`, the CLI MUST output the lifecycle report and discovered detection config as JSON to stdout. Without `--json`, it MUST output a human-readable phase-by-phase summary.

**Manifest Reading**

- **FR-019**: System MUST read raw TOML manifests directly from the manifests repository (not the compiled catalog), since the workflow operates on source manifests. The workflow MUST clone the manifests repo via `actions/checkout` with the Nightwatch GitHub App token into a subdirectory.
- **FR-020**: System MUST resolve download URLs from the autoupdate URL template with version substitution. When no explicit version is provided, the workflow MUST resolve the latest version from the `versions/{package_id}/` directory in the manifests repo checkout (sorted by semver, highest wins).

**Catalog Schema (cross-repo: nightwatch-astro/astro-up-manifests)**

- **FR-021**: The catalog detection table in the manifests compiler MUST be extended to store all fields supported by the detection config type: file_path, version_regex, product_code, upgrade_code, inf_provider, device_class, inf_name, and recursive fallback chain. Fallback depth MUST be capped at 3 levels (primary + 2 fallbacks).
- **FR-022**: The catalog reader in astro-up-core MUST read all detection fields from the updated v1 schema, including the new columns and `fallback_config` JSON blob.
- **FR-027**: The manifests compiler MUST read all detection fields from manifest TOML `[detection]` sections and serialize them into the catalog database, including recursive fallback configs.
- **FR-028**: The catalog schema stays at version `"1"` — the app is not published yet, so no backward compatibility concern. The detection table is updated in place with the new columns.
- **FR-034**: The recursive fallback chain MUST be serialized as a JSON blob in a single `fallback_config` column in the detection table. The reader deserializes it into the existing recursive DetectionConfig struct. The legacy `fallback_method` and `fallback_path` columns are replaced by `fallback_config`.

**Install Path Ledger**

- **FR-023**: System MUST record the discovered install path in the ledger after every successful install (not just in the lifecycle workflow).
- **FR-024**: Install path resolution MUST follow a fallback chain: detection result install_path > registry InstallLocation > default install directory.

**Dry-Run Mode**

- **FR-025**: Dry-run mode MUST download the package and probe for detection signatures without executing install or uninstall phases.
- **FR-026**: Dry-run mode MUST support PE header inspection for downloaded executables and archive content inspection for compressed packages.

### Key Entities

- **DiscoveryCandidate**: A potential detection signature found during probing — includes the detection method, generated config, confidence level, discovered version, and install path.
- **LifecycleReport**: Phase-by-phase test results for a single package — each phase has pass/fail status, duration, exit code (where applicable), and log excerpts.
- **ManifestReader**: Reads raw TOML package manifests and resolves download URLs from autoupdate config.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Lifecycle test workflow completes successfully for at least 80% of packages that have both install and autoupdate sections in their manifests.
- **SC-002**: At least 80% of auto-generated detection configs correctly identify the installed version without manual tweaks (validated against existing hand-written detection configs as ground truth).
- **SC-003**: Each lifecycle phase completes within 10 minutes per package. Full workflow completes within 15 minutes including setup/teardown.
- **SC-004**: Zero dirty runner incidents — every package is cleaned up after testing, even on failure.
- **SC-005**: Matrix sweep processes at least 10 packages per workflow run without interference between jobs.
- **SC-006**: Auto-created PRs contain valid TOML that the manifests repository accepts without modification.
- **SC-007**: Install path is recorded in the ledger for 100% of successful installs, enabling the backup system to resolve install directory tokens.

## Clarifications

### Session 2026-04-04

- Q: Registry discovery matching strategy? → A: Match both package ID and manifest `name` field against registry DisplayName (case-insensitive substring).
- Q: Matrix sweep trigger mode? → A: Manual-only (workflow_dispatch). No scheduled runs — detection configs rarely change. Maintainer triggers when new packages are added.
- Q: Lifecycle report output format? → A: Job summary (markdown in Actions run page) + JSON artifact upload for programmatic use.

## Assumptions

- CI Windows runners run as Administrator, so elevation requirements are satisfied by default.
- The manifests repository is accessible via the existing cross-repo app token (proven by the release workflow).
- Packages with install sections have valid silent switches — installers that require user interaction despite silent switches will time out and be reported as failures.
- The existing Software struct can deserialize raw TOML manifests (confirmed: serde derives are in place).
- CI runners provide sufficient disk space (~14 GB free) for downloading and installing astrophotography packages (largest known: ~500 MB).
- The ~10 existing hand-written detection configs serve as ground truth for validating discovery accuracy.
- The manifests repository (nightwatch-astro/astro-up-manifests) compiler uses the same Rust toolchain and can be modified as part of this spec.

## Scope Boundaries

### In Scope

- Detection discovery module (blind probing of all 7 detection methods)
- CLI subcommand for lifecycle testing
- CI workflow (single, matrix, dry-run modes)
- TOML manifest reader in core crate
- Catalog reader expansion to support all detection config fields
- Manifests compiler schema expansion (nightwatch-astro/astro-up-manifests) to store all detection fields
- Install path ledger recording after every install
- Cross-repo PR creation to manifests repository

### Out of Scope

- Version checking pipeline changes in the manifests repository
- New detection methods beyond the existing 7
- GUI integration for lifecycle testing (this is a CI/developer tool)
- Automated merge of detection PRs (maintainer review required)
