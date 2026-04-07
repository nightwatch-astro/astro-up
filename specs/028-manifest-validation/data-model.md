# Data Model: Manifest URL Validation & Pipeline Hardening

**Date**: 2026-04-07

## Entities

### `AuditReport`

Aggregate validation results across all manifests.

| Field | Type | Description |
|-------|------|-------------|
| `generated_at` | DateTime | Report generation timestamp |
| `manifests_checked` | u32 | Total manifests processed |
| `manifests_passed` | u32 | Manifests passing all checks |
| `manifests_failed` | u32 | Manifests with at least one failure |
| `manifests_skipped` | u32 | Manifests skipped (rate-limited, etc.) |
| `results` | Vec&lt;PackageValidationResult&gt; | Per-manifest results |
| `summary` | ValidationSummary | Aggregate failure counts |

### `PackageValidationResult`

Complete cross-check result for one manifest.

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Manifest ID |
| `provider` | String | Checkver provider type |
| `status` | ResultStatus | pass, fail, skip |
| `version_discovery` | VersionCheck | Checker's version discovery result |
| `url_reachability` | UrlCheck | Download URL reachability |
| `install_method` | InstallMethodCheck | Declared vs detected comparison |
| `playwright` | Option&lt;PlaywrightCheck&gt; | Playwright cross-check (scrape providers only) |
| `version_precision` | Option&lt;PrecisionCheck&gt; | URL-driven version precision check |

### `VersionCheck`

| Field | Type | Description |
|-------|------|-------------|
| `status` | CheckStatus | pass, fail, skip |
| `version` | Option&lt;String&gt; | Discovered version |
| `error` | Option&lt;String&gt; | Error message if failed |

### `UrlCheck`

| Field | Type | Description |
|-------|------|-------------|
| `status` | CheckStatus | pass, fail, skip |
| `url` | Option&lt;String&gt; | Resolved download URL |
| `http_status` | Option&lt;u16&gt; | HTTP response code |
| `failure_type` | Option&lt;UrlFailureType&gt; | permanent, transient, blocked |
| `method_used` | Option&lt;String&gt; | HEAD, GET+Range, GET |

### `InstallMethodCheck`

| Field | Type | Description |
|-------|------|-------------|
| `status` | CheckStatus | pass, fail, skip |
| `declared` | String | Manifest's install.method |
| `detected` | Option&lt;FileType&gt; | Detected from magic bytes |
| `detected_installer_type` | Option&lt;InstallerType&gt; | Specific framework (Inno, NSIS, etc.) |
| `match_result` | MatchResult | match, mismatch, skipped, detection_failed |

### `PlaywrightCheck`

| Field | Type | Description |
|-------|------|-------------|
| `status` | CheckStatus | pass, fail, skip, timeout |
| `version` | Option&lt;String&gt; | Playwright-discovered version |
| `checker_version` | Option&lt;String&gt; | Checker-discovered version (for comparison) |
| `versions_match` | Option&lt;bool&gt; | Whether versions agree |
| `selector_matched` | bool | Whether CSS selector/regex matched |
| `error` | Option&lt;String&gt; | Error details if failed |

### `PrecisionCheck`

| Field | Type | Description |
|-------|------|-------------|
| `status` | CheckStatus | pass, fail, skip |
| `url_contains_version` | bool | Whether resolved URL embeds version |
| `url_version_segment` | Option&lt;String&gt; | Version string found in URL |
| `discovered_version` | String | Version from checker |
| `comparison_mode` | String | "exact" or "normalized" |

## Enums

### `ResultStatus`
pass | fail | skip

### `CheckStatus`
pass | fail | skip | timeout

### `UrlFailureType`
permanent (404/403/410) | transient (5xx/timeout) | blocked (CDN rejection)

### `FileType`
pe_exe | zip | msi | nsis | inno_setup | unknown

### `InstallerType`
inno_setup | nsis | msi | generic_exe | zip | zip_with_installer

### `MatchResult`
match | mismatch | skipped | detection_failed

## Relationships

```
AuditReport 1──* PackageValidationResult
PackageValidationResult 1──1 VersionCheck
PackageValidationResult 1──1 UrlCheck
PackageValidationResult 1──1 InstallMethodCheck
PackageValidationResult 1──? PlaywrightCheck      (only for scrape providers)
PackageValidationResult 1──? PrecisionCheck        (only when URL embeds version)
```

## Extended Manifest Schema

The existing `Install` struct in `crates/shared/src/manifest.rs` gains a new optional field:

| Field | Type | Description |
|-------|------|-------------|
| `detected_type` | Option&lt;String&gt; | Installer framework detected from binary: inno_setup, nsis, msi, generic_exe, zip, zip_with_installer |

Backward compatible: existing manifests without `detected_type` deserialize as `None`. Populated by the audit tool after file type inspection. Used by astro-up GUI/CLI (deferred) to select correct silent install switches.

## Extended Version File

The existing `VersionEntry` gains a new optional field:

| Field | Type | Description |
|-------|------|-------------|
| `url_status` | Option&lt;String&gt; | "reachable", "unreachable", "unchecked" |

Backward compatible: existing version files without `url_status` deserialize as `None` (= unchecked).
