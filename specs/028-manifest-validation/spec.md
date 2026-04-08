# Feature Specification: Manifest URL Validation & Pipeline Hardening

**Feature Branch**: `028-manifest-validation`
**Created**: 2026-04-07
**Status**: Draft
**Type**: implementation
**Project**: Rust Migration
**Project Number**: 1
**Input**: User description: "Manifest URL validation and checker accuracy audit — fix stale URLs, incorrect version resolution, install method mismatches across all manifests; improve checker/compiler validation; add CI integration and Playwright-based scraping validation"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Interactive Per-Package Validation (Priority: P1)

As a manifest maintainer, I validate each package independently using multiple methods before trusting the pipeline output. For every manifest, I cross-check version discovery, URL reachability, and install method correctness using the checker, direct HTTP probing, and Playwright scraping — treating nothing as correct until independently verified.

**Why this priority**: The pipeline's output cannot be trusted until each package is independently verified. This is the foundation — every other story depends on knowing the ground truth per package.

**Independent Test**: Pick any single manifest. Run the checker, probe the download URL directly, scrape the vendor page with Playwright, and compare all three results. If they agree, the manifest is verified. If they disagree, the manifest needs fixing.

**Acceptance Scenarios**:

1. **Given** a manifest with a `github` provider, **When** the validation runs, **Then** it independently: (a) queries the GitHub Releases API for the latest version, (b) resolves the autoupdate URL template with that version and checks HTTP reachability, (c) downloads the first bytes to verify the file type matches the declared install method — and reports agreement or discrepancy for each check
2. **Given** a manifest with an `html_scrape` provider, **When** the validation runs, **Then** it independently: (a) fetches the page with the checker's HTTP client and applies the regex, (b) fetches the same page with Playwright and applies the same regex, (c) compares both discovered versions — and flags if they differ (indicating JS-dependent content the checker misses)
3. **Given** a manifest with a `browser_scrape` provider, **When** the validation runs, **Then** it independently: (a) renders the page with Playwright (matching anti-detection stealth), applies CSS selector/regex, (b) compares the Playwright-discovered version against the checker's chromiumoxide-based discovery, (c) probes the resolved download URL — and reports match status for each
4. **Given** a manifest with a `redirect` provider, **When** the validation runs, **Then** it independently: (a) follows the redirect chain and extracts the version from the final URL, (b) validates the final URL is reachable (HTTP 200), (c) checks the file type of the download
5. **Given** all ~100 manifests, **When** the full validation suite runs, **Then** it produces a per-package report with columns: manifest ID, provider, checker version, Playwright version (if applicable), versions match, URL status (HTTP code), declared install method, detected file type, methods match — with an overall pass/fail per package

---

### User Story 2 - Fix Every Failing Package (Priority: P1)

As a manifest maintainer, for every package that fails interactive validation, I fix the root cause — stale URL template, broken regex, wrong install method — and re-run validation on that specific package to confirm the fix before moving to the next.

**Why this priority**: Equal priority with validation. Each fix must be verified individually, not batched. A manifest is not "fixed" until it passes all independent checks.

**Independent Test**: Take a manifest that failed validation. Apply the fix. Re-run the same multi-method validation on just that manifest. All checks pass.

**Acceptance Scenarios**:

1. **Given** a manifest with a stale autoupdate URL (e.g., Stellarium missing `qt6-` prefix), **When** the URL template is updated, **Then** re-running validation shows: URL reachable (HTTP 200), version matches, file type matches install method
2. **Given** a manifest with a partial-version regex (e.g., NINA capturing "3.2" instead of "3.2.0.9001"), **When** the regex is updated, **Then** re-running validation shows: full version discovered, download URL with full version resolves, Playwright agrees on the version
3. **Given** a manifest declaring `inno_setup` but the download is a zip, **When** the install method is changed to `zip_wrap`, **Then** re-running validation shows: detected file type = zip, declared method = zip_wrap, match
4. **Given** all manifests are fixed and individually verified, **When** the compiler runs, **Then** `catalog.db` compiles without errors and every package has correct metadata

---

### User Story 3 - URL Reachability Validation in Checker (Priority: P2)

As a pipeline operator, after the initial audit is complete and all manifests are verified, I want the checker to automatically validate download URL reachability on every pipeline run — so future breakage is caught immediately.

**Why this priority**: Prevents regression after the initial cleanup. Without this, the same problems will reappear as vendors change URLs.

**Independent Test**: Run the checker with URL validation enabled against a manifest with a known-good URL and one with a known-bad URL. The checker reports reachability status for each.

**Acceptance Scenarios**:

1. **Given** a manifest where the resolved download URL returns HTTP 200, **When** the checker runs with URL validation, **Then** the version file is written with `url_status: reachable`
2. **Given** a manifest where the resolved URL returns HTTP 404, **When** the checker runs with URL validation, **Then** a warning is logged, the version file includes `url_status: unreachable`, and the manifest is flagged in the summary
3. **Given** a URL that returns a redirect chain ending in 200, **When** URL validation runs, **Then** it is considered reachable (follow redirects)
4. **Given** `--skip-url-validation`, **When** the checker runs, **Then** URL validation is skipped (opt-out for speed)

---

### User Story 4 - Automated Install Method Detection (Priority: P2)

As a pipeline operator, I want the checker to detect actual file types and warn on mismatches on every run — so install method drift is caught before it reaches users.

**Why this priority**: Catches a class of error that causes silent installation failures. Automated detection builds on the manual audit work from US1.

**Independent Test**: Run the checker against a manifest where declared method is `inno_setup` but download is a zip. The mismatch is reported automatically.

**Acceptance Scenarios**:

1. **Given** a manifest declaring `inno_setup` and the download has an Inno Setup signature, **When** detection runs, **Then** no mismatch reported
2. **Given** a manifest declaring `inno_setup` but the download is a zip, **When** detection runs, **Then** mismatch warning: "declared: inno_setup, detected: zip"
3. **Given** `download_only` as install method, **When** detection runs, **Then** skipped (any file type valid)
4. **Given** a download URL behind CDN protection that blocks range requests, **When** detection runs, **Then** it falls back to downloading the first 8KB via full GET

---

### User Story 5 - Playwright as Interactive Validation Oracle (Priority: P3)

As a manifest maintainer, during the audit I use Playwright (via MCP tools) to independently navigate vendor pages and verify the checker's scraping logic — providing a second opinion before trusting the checker's output.

**Why this priority**: Browser-scrape manifests are the most fragile. Playwright provides an independent cross-check during the one-time audit. The existing chromiumoxide scraper remains the production tool.

**Independent Test**: Use MCP Playwright tools to navigate to a browser_scrape manifest's URL, apply the same CSS selector and regex, and compare the discovered version against the checker's result.

**Acceptance Scenarios**:

1. **Given** a `browser_scrape` manifest, **When** Playwright navigates to the URL and applies the selector/regex, **Then** the discovered version is compared against the checker's result
2. **Given** a vendor page has changed structure, **When** Playwright renders the page, **Then** the selector/regex fails to match, indicating the manifest needs updating
3. **Given** an `html_scrape` manifest, **When** Playwright renders the page with JS, **Then** it reveals whether JS rendering produces different content than a plain HTTP fetch (indicating the manifest should be `browser_scrape`)
4. **Given** Playwright and checker disagree on a version, **Then** the manifest is investigated and the root cause fixed before marking it as passing

---

### User Story 6 - CI Pipeline Integration (Priority: P3)

As a pipeline operator, I want manifest validation integrated into CI — so broken manifests in PRs are caught before merge.

**Why this priority**: CI prevents broken manifests from entering the production pipeline. Lower priority because core validation must exist first.

**Independent Test**: Submit a PR with a stale URL. CI fails with a clear error.

**Acceptance Scenarios**:

1. **Given** a PR modifying a manifest, **When** CI runs, **Then** validation checks modified manifests for URL reachability, version discovery, and install method consistency
2. **Given** a manifest with valid data, **When** CI runs, **Then** validation passes
3. **Given** a manifest with a stale URL, **When** CI runs, **Then** validation fails with: manifest ID, URL, HTTP status
4. **Given** a PR not touching manifests, **When** CI runs, **Then** manifest validation is skipped

---

### Edge Cases

- What happens when a vendor's download server is temporarily down (transient 5xx)? Retry with backoff; distinguish transient from permanent failures in the report.
- What happens when a vendor uses Cloudflare protection that blocks HEAD requests? Fall back to GET with `Range: bytes=0-0`, then full GET of first 8KB.
- What happens when a download URL redirects to a login page (soft 200)? Install method detection catches this — HTML doesn't match any binary signature.
- What happens when a GitHub release has no assets matching `asset_filter`? URL validation checks the primary URL; the report notes "no matching assets."
- What happens when Playwright times out on a page? Report timeout, don't mark manifest as broken — chromiumoxide may still work. Flag for manual review.
- What happens when checker and Playwright discover different versions? Flag as critical discrepancy — one of the two methods is wrong, requiring manual investigation.
- What happens when the checker runs 100 manifests with URL validation and Playwright? Concurrency limits and rate limiting prevent overwhelming vendors; scrape-based validations run sequentially per domain.

## Requirements *(mandatory)*

### Functional Requirements

**Interactive Per-Package Validation**

- **FR-001**: The validation tool MUST verify each manifest independently using multiple methods: checker version discovery, direct URL probing, and (for scrape providers) Playwright scraping
- **FR-002**: For each manifest, validation MUST cross-check: (a) discovered version matches across methods, (b) resolved download URL is reachable (HTTP 200 after redirects), (c) downloaded file type matches declared install method
- **FR-003**: Validation MUST produce a per-package structured report with: manifest ID, provider type, checker version, Playwright version (if applicable), versions match (yes/no), URL status (HTTP code), declared install method, detected file type, methods match (yes/no), overall pass/fail
- **FR-004**: Validation MUST support running against a single manifest (`--filter <id>`) for targeted re-verification after fixes
- **FR-005**: Validation MUST support running against all manifests with concurrency controls to avoid overwhelming vendor servers

**URL Reachability**

- **FR-006**: URL validation MUST issue an HTTP HEAD request to the resolved download URL
- **FR-007**: If HEAD returns 405 or times out, MUST fall back to GET with `Range: bytes=0-0`
- **FR-008**: MUST follow redirects (up to 10 hops) when validating URL reachability
- **FR-009**: MUST retry on transient failures (5xx, timeout) with exponential backoff (max 3 retries)
- **FR-010**: MUST distinguish permanent failures (404, 403, 410) from transient failures (5xx, timeout)
- **FR-011**: URL validation MUST be opt-out via `--skip-url-validation` for faster runs
- **FR-012**: Version files MUST include a `url_status` field: `reachable`, `unreachable`, or `unchecked`

**Install Method Detection & Schema Migration**

- **FR-013**: MUST detect file type by downloading the first 512 bytes and checking magic bytes
- **FR-014**: File type detection MUST recognize: PE/MZ (exe), PK (zip), MSI (D0CF11E0), NSIS (Nullsoft signature in PE), Inno Setup (signature in PE overlay)
- **FR-015**: MUST compare detected file type against declared install method and report mismatches
- **FR-015a**: When a PE executable is detected, MUST identify the specific installer framework (Inno Setup, NSIS, or generic exe). The detected framework determines which silent install flags to pass (e.g., `/VERYSILENT` for Inno, `/S` for NSIS). The correct framework MUST be set as `install.method`.
- **FR-015b**: For zip downloads, MUST inspect zip contents to detect if it contains a nested installer (exe/msi inside zip). If found, `install.method` MUST be set to the inner installer's framework (e.g., `inno_setup`) and `install.zip_wrapped` MUST be set to `true`.
- **FR-015c**: The `zip_wrap` install method is REMOVED. It is replaced by the combination of `install.method` (the actual installer framework) + `install.zip_wrapped: bool` (delivery format). Valid install methods: `inno_setup`, `nsis`, `msi`, `exe`, `download_only`. A plain zip with no installer (portable app) uses `method = "download_only"` + `zip_wrapped = true`.
- **FR-015d**: `install.zip_wrapped` defaults to `false`. Only manifests whose download is a zip archive set it to `true`.
- **FR-016**: Install method detection MUST be skipped for `download_only` install methods (unless `zip_wrapped = true`, in which case zip contents are still inspected to detect if the method should be upgraded to a real installer framework)
- **FR-016a**: For `manual` provider manifests, validation MUST perform partial checks only: homepage URL reachability and install method detection (if a download URL exists). Version cross-checks MUST be skipped since manual providers return a placeholder version.
- **FR-017**: When range requests are blocked, MUST fall back to downloading the first 8KB via full GET

**Version Cross-Validation**

- **FR-018**: For `html_scrape` manifests, MUST compare plain HTTP fetch + regex against Playwright fetch + regex to detect JS-dependent content
- **FR-019**: For `browser_scrape` manifests, MUST compare Playwright discovery against chromiumoxide discovery to detect engine-specific rendering differences
- **FR-020**: For `github` and `gitlab` manifests, MUST validate the discovered version against the API response directly (not just trusting the provider)
- **FR-021**: For `redirect` manifests, MUST independently follow the redirect chain and extract the version, comparing against the provider result
- **FR-022**: MUST validate discovered versions match the manifest's declared `version_format` (semver, date, or custom regex)
- **FR-023**: If no `version_format` is declared, MUST attempt semver parsing and report a warning if the version doesn't conform
- **FR-024**: Version comparison MUST use URL-driven precision: if the resolved download URL embeds a version string, the discovered version MUST be precise enough to reconstruct that URL segment exactly (e.g., discovering "3.2" is a failure if the URL contains "3.2.0.9001"). If the URL is version-agnostic (generic path, GitHub asset URL resolved by API), normalized comparison applies (strip `v` prefix, trailing `.0` segments).

**Playwright Validation (Interactive, via MCP)**

- **FR-025**: During the audit phase, Playwright (via MCP Playwright server) MUST be used interactively to navigate vendor pages for `browser_scrape` and `html_scrape` manifests and cross-check the checker's version discovery. Playwright is NOT embedded in the checker or CI — it is an interactive validation oracle only.
- **FR-026**: Playwright validation MUST apply the same CSS selectors and regexes as the manifest defines, to verify the checker's scraping logic matches reality
- **FR-027**: When Playwright and the checker disagree on a version, the manifest MUST be investigated and fixed before being marked as passing
- **FR-028**: Playwright validation is NOT required in CI or the 6-hourly pipeline — the checker's chromiumoxide-based `browser_scrape` provider remains the production scraper

**CI Integration**

- **FR-029**: CI MUST run manifest validation on PRs modifying `manifests/**` or `crates/**`
- **FR-030**: CI validation MUST check: schema validity, URL reachability, install method consistency for modified manifests
- **FR-031**: CI MUST fail the check on errors (warnings are non-blocking)
- **FR-032**: CI MUST provide clear, actionable error messages in GitHub Actions output

**Manifest Fixes**

- **FR-033**: All manifests failing URL validation MUST be fixed (updated templates, correct naming conventions)
- **FR-034**: All manifests with partial-version regexes MUST be updated to capture full version strings
- **FR-035**: All manifests with install method mismatches MUST be corrected
- **FR-036**: Each fix MUST be individually verified by re-running validation on that specific manifest
- **FR-037**: After all fixes, the compiler MUST produce a valid `catalog.db` with all packages present and correct

### Key Entities

- **`PackageValidationResult`**: Complete cross-check result for one manifest — manifest ID, provider type, checker version, Playwright version, versions match, URL HTTP status, declared install method, detected file type, methods match, overall pass/fail, failure details
- **`UrlStatus`**: Reachability state — reachable (HTTP 2xx), unreachable (HTTP 4xx with code), transient_failure (HTTP 5xx), unchecked
- **`FileTypeSignature`**: Detected binary format — pe_exe, zip, msi, nsis, inno_setup, unknown, detection_failed
- **`InstallerFramework`**: Detected installer framework from binary inspection — inno_setup, nsis, msi, exe (generic PE). Stored directly as `install.method`. Combined with `install.zip_wrapped: bool` to indicate delivery format. Replaces the former `zip_wrap` method and `detected_type` field.
- **`VersionCrossCheck`**: Multi-method version comparison — checker version, Playwright version (via MCP), API version (for github/gitlab), all match (yes/no), discrepancy details
- **`ValidationSummary`**: Aggregate across all manifests — total checked, passed, failed by category (URL, version, install method, scrape discrepancy), list of actionable fixes

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of manifests pass the full interactive validation after fixes — zero stale URLs, zero version mismatches, zero install method mismatches
- **SC-002**: Every individual manifest fix is verified by re-running validation on that specific package before moving to the next
- **SC-003**: The checker discovers the correct current version for at least 95% of non-manual manifests (5% allowance for vendors with temporary issues)
- **SC-004**: Playwright version discovery agrees with the checker for at least 90% of scrape-based manifests
- **SC-005**: Install method detection correctly identifies file types for at least 98% of downloads
- **SC-006**: URL reachability validation adds less than 30 seconds to a full pipeline run (with concurrency)
- **SC-007**: CI validation catches broken manifests in PRs within 5 minutes of push
- **SC-008**: The compiled `catalog.db` is valid and complete after all fixes
- **SC-009**: Zero regression — no previously-working manifest breaks as a result of changes

## Clarifications

### Session 2026-04-07

- Q: When cross-checking versions between methods, what counts as "the same version"? → A: URL-driven precision — if the download URL embeds a version string, the discovered version must match it exactly (partial match = regex is wrong). If the URL is generic/version-agnostic, normalized comparison applies (strip `v` prefix, trailing `.0` segments).
- Q: How should `manual` provider manifests be handled during validation? → A: Partial validation only — check homepage URL reachability and install method detection if a download URL exists, but skip version cross-checks.
- Q: Should detected installer type be captured for future use? → A: Yes. Remove `zip_wrap` as a method. Split into `install.method` (real framework: inno_setup, nsis, msi, exe, download_only) + `install.zip_wrapped: bool` (delivery format, default false). Plain zip = `download_only` + `zip_wrapped = true`. Create deferred issues for GUI/CLI to handle both fields.
- Q: Is Playwright embedded in the checker or CI? → A: No. Playwright is used interactively via MCP tools during the one-time audit to cross-check the checker's scraping. The existing chromiumoxide browser_scrape provider remains the production scraper. No Node.js dependency in the pipeline.

## Assumptions

- The `astro-up-manifests` repo is at `../astro-up-manifests` (sibling directory) with write access
- Vendor download servers are generally available; transient failures (< 5 minutes) are tolerable
- The existing checker provider architecture (9 providers) is stable — this spec extends it
- GitHub API rate limits (5000 req/hr) are sufficient for checking all GitHub-based manifests
- Playwright is available locally via MCP server for interactive validation — NOT required in CI
- The chromiumoxide-based browser_scrape provider is the production scraper; Playwright is an interactive validation oracle only
- PR #150 (prefer GitHub asset URLs over templates) is merged and stable
- The `checker-state.json` and version file formats can be extended with new fields without breaking existing pipeline state
- Interactive validation runs locally (not in the 6-hourly pipeline) — the pipeline gains URL validation and install method detection, but not full Playwright cross-checks on every run
