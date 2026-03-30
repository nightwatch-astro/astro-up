# Feature Specification: Remote Version Providers

**Feature Branch**: `008-remote-providers`
**Created**: 2026-03-29
**Status**: Deferred
**Deferred Reason**: Client relies on catalog for version info. Deferred with spec 014 (Custom Tools)
**Project**: Rust Migration
**Project Number**: 1
**Input**: Migration plan Spec 007 — check latest versions from GitHub, GitLab, and vendor websites

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Check Latest Version from GitHub (Priority: P1)

A user runs `astro-up check` and the application queries GitHub Releases for each package that uses the GitHub provider. It retrieves the latest release version, comparing it against the installed version to determine if an update is available.

**Why this priority**: GitHub Releases is the most common distribution channel for astro software (~60% of packages).

**Independent Test**: Query NINA's GitHub releases, verify the latest version is correctly parsed.

**Acceptance Scenarios**:

1. **Given** a package with `checkver.github = "isbeorn/nina"`, **When** checking for updates, **Then** the latest GitHub release version is retrieved
2. **Given** a GitHub rate limit is hit, **When** checking, **Then** the system backs off and retries or reports the rate limit
3. **Given** a GitHub token is configured, **When** checking, **Then** authenticated requests are used (higher rate limit)

---

### User Story 2 - Check Version from Vendor Website (Priority: P2)

For packages not on GitHub, the application scrapes the vendor's website using configurable URL + regex or URL + CSS selector patterns to extract the latest version string.

**Why this priority**: ~30% of astro software is distributed via vendor websites (PHD2, Stellarium, etc.).

**Independent Test**: Query a vendor URL with a version regex, verify the correct version is extracted.

**Acceptance Scenarios**:

1. **Given** a package with `checkver.url` and `checkver.regex`, **When** checking, **Then** the version is extracted from the page
2. **Given** the vendor website is down, **When** checking, **Then** a timeout error is reported for that package
3. **Given** the regex doesn't match, **When** checking, **Then** a "version not found" error is reported with the URL

---

### User Story 3 - Check Version from GitLab (Priority: P3)

The application queries GitLab Tags API for packages hosted on GitLab (e.g., some INDI drivers).

**Why this priority**: Smaller package count but important for completeness.

**Independent Test**: Query a GitLab repo's tags, verify the latest version tag is parsed.

**Acceptance Scenarios**:

1. **Given** a package with GitLab provider config, **When** checking, **Then** the latest tag version is returned
2. **Given** pre-release tags exist, **When** checking with stable channel, **Then** pre-releases are filtered out

### Edge Cases

- API returns paginated results: Follow pagination to find the true latest.
- Version tag has a prefix (e.g., "Version-3.1.2"): Strip the configured `tag_prefix`.
- Multiple assets per release: Filter by `asset_pattern` glob.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST check latest versions via GitHub Releases API
- **FR-002**: System MUST check latest versions via GitLab Tags API
- **FR-003**: System MUST check latest versions via URL + regex extraction (direct URL provider)
- **FR-004**: System MUST check latest versions via URL + CSS selector (HTML scrape provider)
- **FR-005**: System MUST check latest versions via HTTP HEAD response headers (http_head provider)
- **FR-006**: System MUST support declarative checkver patterns from manifest TOML
- **FR-007**: System MUST respect GitHub API rate limits and use token-authenticated requests when configured
- **FR-008**: System MUST support `tag_prefix` stripping for version tag parsing
- **FR-009**: System MUST support `asset_pattern` glob matching for release asset filtering
- **FR-010**: System MUST report per-package check results (success with version, or error with reason)
- **FR-011**: System MUST support configurable request timeout per provider (default: 30 seconds)
- **FR-012**: System MUST support the `$version` template variable in autoupdate URLs

### Key Entities

- **CheckResult**: Success(latest_version, download_url, changelog_url) or Error(reason)
- **ProviderConfig**: Union of GitHub, GitLab, DirectUrl, HtmlScrape, HttpHead configurations
- **RateLimiter**: Tracks API call budgets per provider to avoid rate limiting

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Version check completes for all ~95 packages in under 60 seconds (with parallelism)
- **SC-002**: GitHub provider correctly parses versions from 100% of test repos
- **SC-003**: Rate limiting prevents 429 errors during bulk checks

## Assumptions

- GitHub API token is optional but recommended for higher rate limits
- HTML scraping is inherently fragile — the manifest repo CI validates scrape patterns weekly
- Browser-based scraping (headless Chrome) is NOT in this spec — it stays in the manifest repo checker
- Depends on: spec 003 (types), spec 004 (config for tokens/timeouts), spec 005 (catalog for manifest data)

## Clarifications

- **Provider selection is manifest-driven**: Each manifest declares its `[checkver]` section. The client dispatches to the matching provider. No auto-detection.
- **GitHub provider shorthand**: `checkver.github = "owner/repo"` is equivalent to a full config with GitHub Releases API, `tag_prefix` stripping, and `asset_pattern` matching.
- **Rate limiting strategy**: Per-host token bucket. GitHub: 60/hour unauthenticated, 5000/hour with token. GitLab: 300/min. Custom hosts: 10/min default. Token refills continuously, not per-window.
- **Parallel checking with concurrency limit**: Check packages in parallel (default: 10 concurrent). Rate limiter gates per-host. Total wall time for ~95 packages: under 60s with token.
- **No browser scraping in client**: HTML scraping uses reqwest + scraper (CSS selectors). JavaScript-heavy sites are handled by the manifest repo's CI checker, not the client.
- **Autoupdate URL templates**: After discovering a new version, construct the download URL using `$version`, `$majorVersion`, `$cleanVersion` etc. Template expansion happens in this spec, feeding into spec 010 (download).
- **Failed checks don't block**: Each package check is independent. Failures are collected and reported at the end. The user sees "3 of 95 checks failed" not a crash.
