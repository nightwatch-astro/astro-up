# Implementation Plan: Manifest URL Validation & Pipeline Hardening

**Branch**: `028-manifest-validation` | **Date**: 2026-04-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/028-manifest-validation/spec.md`

## Summary

Audit and harden the astro-up-manifests pipeline by: (1) adding file type detection, URL reachability validation, and installer type capture to the checker, (2) interactively validating every manifest using the checker + MCP Playwright, (3) fixing all broken manifests, (4) adding CI validation. All code changes are in `../astro-up-manifests`.

## Technical Context

**Language/Version**: Rust 2024 edition (1.85+), workspace in `../astro-up-manifests`
**Primary Dependencies**: reqwest 0.13, pelite 0.10, tokio 1, serde/serde_json 1, chromiumoxide 0.8, clap 4
**Storage**: JSON version files, SQLite catalog.db (rusqlite 0.39 bundled)
**Testing**: cargo nextest, insta for snapshots
**Target Platform**: Linux (CI), macOS (local dev)
**Project Type**: CLI tools (checker, compiler) + manifest data files
**Performance Goals**: URL validation adds <30s to full pipeline run with concurrency
**Constraints**: GitHub API rate limit 5000 req/hr, vendor server rate limiting
**Scale/Scope**: ~95 manifest TOML files, 9 provider types

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | All new code in existing crates (shared, checker) — no new crates |
| II. Platform Awareness | PASS | No Windows-specific code; file type detection uses cross-platform `pelite` |
| III. Test-First | PASS | Integration tests for file detection, URL validation; insta snapshots for audit reports |
| IV. Thin Tauri Boundary | N/A | No GUI changes in this spec |
| V. Spec-Driven | PASS | This is the spec |
| VI. Simplicity | PASS | Extends existing checker with new flags, no new binaries or frameworks |
| VII. Observability | PASS | Structured logging for URL validation, file detection, audit results |

## Project Structure

### Documentation (this feature)

```text
specs/028-manifest-validation/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── contracts/
│   ├── audit-report.json    # JSON schema for audit output
│   └── playwright-validate.md  # MCP Playwright validation workflow
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (in ../astro-up-manifests)

```text
crates/
  shared/src/
    file_type.rs          # NEW — magic bytes detection + installer type identification
    manifest.rs           # MODIFY — add detected_type to Install struct
    version_file.rs       # MODIFY — add url_status field to VersionEntry
    validate.rs           # MODIFY — add detected_type to known fields
  checker/src/
    main.rs               # MODIFY — add --audit flag, --skip-url-validation
    audit.rs              # NEW — audit mode orchestration, report generation
    url_validate.rs       # NEW — HEAD/GET fallback URL reachability check
    version_writer.rs     # MODIFY — write url_status to version files
  compiler/src/
    compile.rs            # MODIFY — compile detected_type into catalog.db
    schema.rs             # MODIFY — add detected_type column to install table

manifests/*.toml          # MODIFY — fix broken manifests, add detected_type

.github/workflows/
  ci.yml                  # MODIFY — add manifest validation job

tests/
  file_type_detection.rs  # NEW — magic bytes test fixtures
  audit_integration.rs    # NEW — full audit mode test with sample manifests
```

**Structure Decision**: All changes in existing crates. No new crates, no Node.js, no Playwright dependency. File type detection in shared (reusable), audit orchestration in checker.

## Complexity Tracking

No constitution violations — no complexity justification needed.

## Phase Implementation

### Phase A: Foundation (shared crate changes)

1. **File type detection module** (`crates/shared/src/file_type.rs`)
   - `detect_file_type(bytes: &[u8]) -> FileType` — magic bytes at offset 0
   - `detect_installer_type(bytes: &[u8]) -> InstallerType` — PE overlay scanning via pelite
   - For ZIP: `detect_zip_contents(bytes: &[u8]) -> InstallerType` — check if zip contains exe/msi
   - Test fixtures: sample headers for each type (Inno, NSIS, MSI, ZIP, generic PE)

2. **Manifest schema extension** (`crates/shared/src/manifest.rs`)
   - Add `detected_type: Option<String>` to `Install` struct
   - Serde default = None, skip_serializing_if None

3. **Version file extension** (`crates/shared/src/version_file.rs`)
   - Add `url_status: Option<String>` to `VersionEntry`
   - Values: "reachable", "unreachable", "unchecked"
   - Serde default = None, skip_serializing_if None

### Phase B: Checker Extensions

4. **URL reachability module** (`crates/checker/src/url_validate.rs`)
   - `validate_url(client: &RetryClient, url: &str) -> UrlCheck`
   - Fallback chain: HEAD → GET+Range → GET first 8KB
   - Returns: status, http_code, failure_type, downloaded_bytes (reused for file detection)
   - Respects existing retry/backoff from `RetryClient`

5. **Audit mode** (`crates/checker/src/audit.rs`)
   - New `--audit` CLI flag → enables URL validation + file type detection + JSON report
   - Orchestrates per-manifest: version discovery → URL validation → file type detection → version precision check
   - Produces `AuditReport` JSON to stdout, human summary to stderr
   - `--filter <id>` works with audit mode for single-manifest re-verification
   - `--skip-url-validation` opt-out flag

6. **Version precision check** (in `audit.rs`)
   - After template substitution, check if resolved URL contains `$version` literally
   - If yes → exact string comparison
   - If no → normalized comparison (strip v prefix, trailing .0)

### Phase C: Interactive Audit & Fixes

7. **Run full audit** — execute checker `--audit` against all manifests
8. **Playwright cross-check** — for each scrape-based manifest, use MCP Playwright to verify
9. **Fix manifests** — update URL templates, regexes, install methods, add detected_type
10. **Re-verify each fix** — run `--audit --filter <id>` per fixed manifest
11. **Compile catalog** — run compiler, verify catalog.db is valid

### Phase D: CI & Deferred Issues

12. **CI validation job** — add to `.github/workflows/ci.yml`:
    - Path-conditional: only on `manifests/**` or `crates/**` changes
    - Runs: `cargo run -p astro-up-checker -- --audit --manifests manifests`
    - Fails on errors, warns on warnings

13. **Compiler update** — add `detected_type` column to install table in schema.rs

14. **Deferred issues** — create GitHub issues for:
    - astro-up CLI: consume `install.detected_type` for silent install flag selection
    - astro-up GUI: display detected installer type, use for install flags
