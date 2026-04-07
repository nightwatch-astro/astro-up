# Research: Manifest URL Validation & Pipeline Hardening

**Date**: 2026-04-07

## R1: Playwright Integration Approach

**Decision**: Interactive validation via MCP Playwright server — NOT embedded in checker or CI

**Rationale**: Playwright serves as an independent validation oracle during the one-time audit. The maintainer uses MCP Playwright tools to navigate vendor pages and cross-check the checker's version discovery. The existing chromiumoxide-based `browser_scrape` provider remains the production scraper in the 6-hourly pipeline. This avoids adding Node.js as a dependency to the Rust pipeline.

**Alternatives considered**:
- Standalone Node.js script (`scripts/playwright-validate.js`) — adds Node.js dependency, maintenance burden, CI complexity
- Replace chromiumoxide with Playwright entirely — breaks existing pipeline, high risk
- Automated Playwright in CI — expensive (browser install), slow, not needed for ongoing pipeline

## R2: File Type Detection via Magic Bytes

**Decision**: New `file_type` module in `crates/shared/src/file_type.rs`

**Rationale**: Detecting installer type from file headers is well-established. The checker downloads the first 512 bytes (or 8KB fallback for CDNs blocking range requests) and matches against known signatures.

**Signatures**:

| Type | Magic Bytes | Offset | Notes |
|------|------------|--------|-------|
| PE/MZ (exe) | `4D 5A` | 0 | DOS MZ header |
| ZIP | `50 4B 03 04` | 0 | PK local file header |
| MSI | `D0 CF 11 E0 A1 B1 1A E1` | 0 | OLE Compound Document |
| NSIS | "NullsoftInst" in PE overlay | variable | Search PE overlay for `4E 75 6C 6C 73 6F 66 74 49 6E 73 74` |
| Inno Setup | "Inno Setup Setup Data" in PE overlay | variable | Search after PE sections end |

**Detection strategy**:
1. Check magic bytes at offset 0 → ZIP, MSI, or PE
2. If PE: parse with `pelite` (already a workspace dependency), scan overlay data for NSIS/Inno signatures
3. If neither found: classify as `generic_exe`
4. For ZIP: optionally inspect zip central directory for nested installers

## R3: Installer Type Capture

**Decision**: New `install.detected_type` field in manifest TOML schema

**Rationale**: The current `install.method` field describes *how* to install (inno_setup, msi, nsis, zip_wrap, exe, download_only) but for `exe` and `zip_wrap` methods, it doesn't capture the specific installer framework. Knowing the framework is essential for passing correct silent install flags. Detecting and storing this during the audit provides ground truth for future GUI/CLI install flag logic.

**Values**: `inno_setup`, `nsis`, `msi`, `generic_exe`, `zip`, `zip_with_installer`

**Manifest schema change** (crates/shared/src/manifest.rs `Install` struct):
- Add `detected_type: Option<String>` — optional, populated by audit, not required for existing manifests
- Backward compatible: missing field deserializes as `None`

**Deferred issues needed**:
- astro-up CLI: Use `install.detected_type` to select correct silent install switches
- astro-up GUI: Display detected installer type in package detail view, use for install flag selection

## R4: URL Reachability Validation Strategy

**Decision**: HTTP HEAD with GET fallback, integrated into checker's `process_manifest` flow

**Rationale**: HEAD is cheapest. Some CDNs reject HEAD or return different status. Fallback chain: HEAD → GET with `Range: bytes=0-0` → GET first 8KB. The Range GET response serves double duty — validates URL reachability AND provides bytes for file type detection. One request, two checks.

## R5: Version Comparison with URL-Driven Precision

**Decision**: Context-dependent comparison in checker's audit mode

**Heuristic**: After template substitution, check if `$version` appears literally in the resolved URL. If yes → exact comparison required. If no (URL was generic or asset-resolved) → normalize and compare (strip `v` prefix, trailing `.0` segments).

## R6: Audit Report Structure

**Decision**: JSON report to stdout (machine-readable) with human-readable summary to stderr

**Format**: See `contracts/audit-report.json` for the JSON schema.
