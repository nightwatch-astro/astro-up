# Tasks: Manifest URL Validation & Pipeline Hardening

**Input**: Design documents from `/specs/028-manifest-validation/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/
**Target repo**: `../astro-up-manifests` (all code changes)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to

---

## Phase 1: Foundational (Data Model & Detection)

**Purpose**: Core types and file detection module that ALL user stories depend on

- [ ] T001 [P] Create audit data model types (enums + structs) in `crates/shared/src/audit_types.rs` — `ResultStatus`, `CheckStatus`, `UrlFailureType`, `FileType`, `InstallerType`, `MatchResult`, `UrlCheck`, `VersionCheck`, `InstallMethodCheck`, `PrecisionCheck`, `PackageValidationResult`, `AuditReport`, `ValidationSummary` per data-model.md
- [ ] T002 [P] Implement file type detection module in `crates/shared/src/file_type.rs` — `detect_file_type(bytes: &[u8]) -> FileType` (magic bytes: MZ=pe_exe, PK=zip, D0CF11E0=msi), `detect_installer_type(bytes: &[u8]) -> InstallerType` (pelite PE overlay scan for "NullsoftInst"=nsis, "Inno Setup"=inno_setup, else generic_exe), `detect_zip_contents(bytes: &[u8]) -> InstallerType` (check zip central directory for exe/msi)
- [ ] T003 [P] Add `detected_type: Option<String>` to `Install` struct in `crates/shared/src/manifest.rs` — serde default None, skip_serializing_if Option::is_none. Add to `KNOWN_DETECTED_TYPES` const in `crates/shared/src/validate.rs`
- [ ] T004 [P] Add `url_status: Option<String>` to `VersionEntry` struct in `crates/shared/src/version_file.rs` — serde default None, skip_serializing_if Option::is_none. Values: "reachable", "unreachable", "unchecked"
- [ ] T005 Wire new modules into `crates/shared/src/lib.rs` — add `pub mod audit_types;` and `pub mod file_type;`
- [ ] T006 Add tests for file type detection in `crates/shared/tests/file_type_detection.rs` — test fixtures with known magic bytes for PE, ZIP, MSI, NSIS (with Nullsoft string), Inno Setup (with signature), and unknown format

**Checkpoint**: Foundation ready — shared types and detection logic available for checker

---

## Phase 2: User Story 1 — Interactive Per-Package Validation (Priority: P1) 🎯 MVP

**Goal**: `--audit` mode that validates each manifest independently: version discovery → URL reachability → file type detection → version precision → structured JSON report

**Independent Test**: `cargo run -p astro-up-checker -- --audit --filter nina-app --manifests manifests --versions versions` produces a JSON report with pass/fail per check

- [ ] T007 [US1] Implement URL reachability module in `crates/checker/src/url_validate.rs` — `validate_url(client: &RetryClient, url: &str) -> UrlCheck` with fallback chain: HEAD → GET+Range:bytes=0-0 (on 405/timeout) → GET first 8KB (on blocked). Follow redirects up to 10 hops. Retry transient 5xx with backoff (reuse RetryClient). Return UrlCheck with status, http_code, failure_type, downloaded_bytes (for file detection reuse)
- [ ] T008 [US1] Implement version precision and format validation in `crates/checker/src/version_precision.rs` — (a) `check_precision(resolved_url: &str, version: &str, template: Option<&str>) -> PrecisionCheck`: if template contains `$version` and resolved URL embeds version → exact match required; if URL is generic → normalized comparison (strip `v` prefix, trailing `.0`). (b) `validate_version_format(version: &str, version_format: Option<&str>) -> CheckStatus`: if version_format is "semver" → parse as semver; if "date" → parse as date; if custom regex → match against it. If no version_format declared → attempt semver, warn if non-conforming. Return CheckStatus. Covers FR-022, FR-023, FR-024
- [ ] T009 [US1] Implement audit mode orchestration in `crates/checker/src/audit.rs` — `run_audit(manifests: &[Manifest], client: &RetryClient, versions_dir: &Path, skip_url_validation: bool) -> AuditReport`. Per manifest: (1) run existing provider check, (2) resolve download URL, (3) if !skip_url_validation: validate URL + get bytes, (4) detect file type + installer type from bytes, (5) compare declared vs detected install method, (6) check version precision, (7) build PackageValidationResult. For manual providers: partial checks only (homepage URL + install method if URL exists). Aggregate into AuditReport. JSON to stdout, human summary to stderr
- [ ] T010 [US1] Add `--audit` and `--skip-url-validation` CLI flags to `crates/checker/src/main.rs` — new Cli fields. When `--audit` is set, call `audit::run_audit()` instead of the normal `process_manifest` loop. `--filter` continues to work in audit mode. Print JSON report to stdout, summary to stderr
- [ ] T011 [US1] Wire new modules into `crates/checker/src/lib.rs` — add `pub mod audit;`, `pub mod url_validate;`, `pub mod version_precision;`
- [ ] T012 [US1] Add integration test for audit mode in `crates/checker/tests/audit_integration.rs` — create 3 sample manifests (one valid github, one with stale URL, one with install method mismatch). Run audit. Assert: valid manifest passes all checks, stale URL fails url_reachability, mismatch fails install_method. Verify JSON report structure matches contracts/audit-report.json schema

**Checkpoint**: `cargo run -p astro-up-checker -- --audit` produces a full validation report

---

## Phase 3: User Story 2 — Fix Every Failing Package (Priority: P1)

**Goal**: Run the audit against all ~95 manifests, fix every failure, re-verify each fix individually, compile valid catalog.db

**Independent Test**: After all fixes, `cargo run -p astro-up-checker -- --audit` reports 0 failures and `cargo run -p astro-up-compiler -- --manifests manifests --versions versions --output catalog.db` succeeds

- [ ] T013 [US2] Run full audit against all manifests — `cargo run -p astro-up-checker -- --audit --manifests manifests --versions versions 2>audit-stderr.log >audit-report.json`. Triage the JSON report: categorize failures by type (URL 404, version mismatch, install method mismatch, version precision). Document findings in a triage file
- [ ] T014 [US2] Fix manifests with stale autoupdate URL templates in `manifests/*.toml` — update URL templates to match current vendor naming conventions (e.g., Stellarium qt6- prefix, NINA zip URL). Re-verify each fix: `cargo run -p astro-up-checker -- --audit --filter <id>`
- [ ] T015 [US2] Fix manifests with partial-version regexes in `manifests/*.toml` — update regexes to capture full version strings. Re-verify each fix individually
- [ ] T016 [US2] Fix manifests with install method mismatches in `manifests/*.toml` — correct declared install method to match detected file type (e.g., inno_setup→zip_wrap). Re-verify each fix individually
- [ ] T017 [US2] Add `detected_type` to all manifests in `manifests/*.toml` — based on audit file type detection results, add `detected_type = "inno_setup"` (or nsis, msi, generic_exe, zip, zip_with_installer) to each manifest's `[install]` section. Re-verify each manifest individually
- [ ] T018 [US2] Run final full audit and compile catalog — `cargo run -p astro-up-checker -- --audit` must report 0 errors. `cargo run -p astro-up-compiler -- --manifests manifests --versions versions --output catalog.db` must succeed. Verify catalog.db has all packages with correct metadata

**Checkpoint**: All manifests pass audit, catalog.db compiles successfully

---

## Phase 4: User Story 3 — URL Reachability in Normal Checker (Priority: P2)

**Goal**: The normal (non-audit) checker flow writes `url_status` to version files on every run

**Independent Test**: `cargo run -p astro-up-checker -- --manifests manifests --versions versions` writes version files with `url_status: reachable` or `url_status: unreachable`

- [ ] T019 [US3] Integrate URL validation into normal checker flow in `crates/checker/src/main.rs` — after `handle_found()` resolves the download URL, call `url_validate::validate_url()` (reusing the module from T007). Set `url_status` on the `DiscoveredVersion` before writing. Respect `--skip-url-validation` flag. Log warnings for unreachable URLs
- [ ] T020 [US3] Update `DiscoveredVersion` in `crates/checker/src/version_writer.rs` — add `url_status: Option<String>` field. Pass through to `VersionEntry` on write

**Checkpoint**: Normal checker runs include URL reachability in version files

---

## Phase 5: User Story 4 — Automated Install Method Detection (Priority: P2)

**Goal**: The normal checker flow detects file types and warns on mismatches on every run

**Independent Test**: Run checker against a manifest with wrong install method → mismatch warning logged

- [ ] T021 [P] [US4] Integrate install method detection into normal checker flow in `crates/checker/src/main.rs` — after URL validation (which provides downloaded_bytes), call `file_type::detect_file_type()` and `file_type::detect_installer_type()`. Compare against `manifest.install.method`. Log warning on mismatch. Skip for `download_only`. Log detected installer type at debug level
- [ ] T022 [P] [US4] Add install method mismatch to checker summary in `crates/checker/src/main.rs` — extend `Summary` struct with `method_mismatches: Vec<String>`. Print mismatches in `print_summary()`

**Checkpoint**: Normal checker runs detect and warn on install method mismatches

---

## Phase 6: User Story 5 — Playwright Interactive Validation (Priority: P3) [MANUAL]

**Goal**: Use MCP Playwright tools to cross-check scrape-based manifests during the audit

**Independent Test**: Navigate to a browser_scrape manifest's URL via MCP Playwright, apply selector/regex, compare version against checker result

- [ ] T023 [US5] [MANUAL] Validate all `browser_scrape` manifests via MCP Playwright — for each manifest: navigate to URL with anti-detection stealth, apply CSS selector and regex, compare discovered version against checker audit result. Fix any discrepancies found. Document results
- [ ] T024 [US5] [MANUAL] Validate all `html_scrape` manifests via MCP Playwright — for each manifest: render page with JS via Playwright, apply regex, compare against checker's plain HTTP fetch result. Identify manifests that should be `browser_scrape` (JS-dependent content). Fix and re-verify

**Checkpoint**: All scrape-based manifests verified via independent Playwright cross-check

---

## Phase 7: User Story 6 — CI Pipeline Integration (Priority: P3)

**Goal**: CI validates manifests on PRs — schema validity, URL reachability, install method consistency

**Independent Test**: Submit a PR with a stale URL manifest → CI fails with actionable error

- [ ] T025 [US6] Add manifest validation job to `.github/workflows/ci.yml` — path-conditional on `manifests/**` or `crates/**` changes. Run `cargo run -p astro-up-checker -- --audit --manifests manifests --versions versions`. Fail on errors (exit code 1 from checker on failures). Use existing sccache and Rust toolchain setup
- [ ] T026 [US6] Update checker audit exit codes in `crates/checker/src/audit.rs` — exit 0 on all pass, exit 1 on any error-severity failure, exit 0 with warnings (non-blocking). Ensure error messages are clear in GitHub Actions output (no ANSI codes, include manifest ID and failure type)

**Checkpoint**: CI catches broken manifests in PRs

---

## Phase 8: Polish & Cross-Cutting

**Purpose**: Compiler schema update, deferred issues, documentation

- [ ] T027 [P] Update compiler schema in `crates/compiler/src/schema.rs` — add `detected_type TEXT` column to `install` table
- [ ] T028 [P] Update compiler to write `detected_type` in `crates/compiler/src/compile.rs` — read `manifest.install.detected_type` and insert into the new column
- [ ] T029 Final compile and validate — run compiler on all manifests, verify `catalog.db` has `detected_type` populated for all packages. Run `cargo test --workspace` and `cargo clippy --workspace -- -D warnings`
- [ ] T030 [P] Create deferred GitHub issue: astro-up CLI to consume `install.detected_type` for silent install switch selection — label `deferred`, `spec:028`, `spec:015-cli-interface`
- [ ] T031 [P] Create deferred GitHub issue: astro-up GUI to display detected installer type and use for install flag selection — label `deferred`, `spec:028`, `spec:016-tauri-app-shell`

---

## Task Dependencies

<!-- Machine-readable. Generated by /speckit.tasks, updated by /speckit.iterate.apply -->
<!-- Do not edit manually unless you also update GitHub issue dependencies -->

```toml
[graph]
# Phase 1: Foundational — no blockers except internal sequencing
[graph.T001]
blocked_by = []

[graph.T002]
blocked_by = []

[graph.T003]
blocked_by = []

[graph.T004]
blocked_by = []

[graph.T005]
blocked_by = ["T001", "T002"]

[graph.T006]
blocked_by = ["T002"]

# Phase 2: US1 — depends on foundation
[graph.T007]
blocked_by = ["T001", "T004"]

[graph.T008]
blocked_by = ["T001"]

[graph.T009]
blocked_by = ["T001", "T002", "T005", "T007", "T008"]

[graph.T010]
blocked_by = ["T009"]

[graph.T011]
blocked_by = ["T007", "T008", "T009"]

[graph.T012]
blocked_by = ["T010"]

# Phase 3: US2 — depends on audit mode working
[graph.T013]
blocked_by = ["T010"]

[graph.T014]
blocked_by = ["T013"]

[graph.T015]
blocked_by = ["T013"]

[graph.T016]
blocked_by = ["T013"]

[graph.T017]
blocked_by = ["T013", "T003"]

[graph.T018]
blocked_by = ["T014", "T015", "T016", "T017"]

# Phase 4: US3 — depends on url_validate module
[graph.T019]
blocked_by = ["T007", "T004", "T011"]

[graph.T020]
blocked_by = ["T004"]

# Phase 5: US4 — depends on file_type module
[graph.T021]
blocked_by = ["T002", "T005", "T007"]

[graph.T022]
blocked_by = ["T021"]

# Phase 6: US5 — depends on audit results
[graph.T023]
blocked_by = ["T013"]

[graph.T024]
blocked_by = ["T013"]

# Phase 7: US6 — depends on audit exit codes
[graph.T025]
blocked_by = ["T010"]

[graph.T026]
blocked_by = ["T009"]

# Phase 8: Polish
[graph.T027]
blocked_by = ["T003"]

[graph.T028]
blocked_by = ["T027"]

[graph.T029]
blocked_by = ["T018", "T028"]

[graph.T030]
blocked_by = []

[graph.T031]
blocked_by = []
```

---

## Parallel Opportunities

### Phase 1 (Foundation)
```
Parallel: T001, T002, T003, T004 (all independent, different files)
Then: T005 (wires modules), T006 (tests)
```

### Phase 2 (US1)
```
Parallel: T007, T008 (url_validate.rs and version_precision.rs are independent)
Then: T009 (audit.rs orchestrates both)
Then: T010 (CLI wiring)
Parallel: T011 (lib.rs wiring), T012 (integration test)
```

### Phase 3 (US2 — fixes)
```
After T013 (triage):
  Parallel: T014, T015, T016, T017 (different manifests, independent fix types)
Then: T018 (final verify + compile)
```

### Phase 4+5 (US3+US4 — can run parallel with US2)
```
Parallel: T019+T020 (US3) and T021+T022 (US4) — different files
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Foundation (T001–T006)
2. Complete Phase 2: US1 audit mode (T007–T012)
3. **STOP and VALIDATE**: Run `--audit --filter <some-manifest>` — verify JSON report
4. This gives you the validation tool to assess the full manifest set

### Incremental Delivery

1. Foundation → US1 audit mode → **validation tool ready**
2. US2 manifest fixes → **all manifests passing**
3. US3 + US4 (parallel) → **automated regression detection**
4. US5 Playwright cross-check → **scraping confidence**
5. US6 CI integration → **PR-level protection**
6. Polish → **compiler schema, deferred issues**

---

## Notes

- All file paths are relative to `../astro-up-manifests` (sibling repo)
- T023 and T024 (Playwright) are [MANUAL] — require interactive MCP tools, cannot be automated
- T014–T017 (manifest fixes) will be the bulk of the work — ~95 manifests to audit and potentially fix
- T030 and T031 (deferred issues) have no blockers and can be created at any time
- The audit mode (US1) reuses existing provider logic — it adds validation layers on top, doesn't replace anything
