# Research: 023-lifecycle-testing

## pelite — Cross-Platform PE Parsing

**Decision**: Use pelite 0.10 (already a dependency)
**Rationale**: Confirmed cross-platform — works on macOS/Linux for dry-run mode. Already integrated in `detect/pe.rs`. Supports both `VS_FIXEDFILEINFO` (binary version) and `StringFileInfo` (ProductName, CompanyName) for discovery matching.
**Alternatives considered**: None — pelite is the established choice in this codebase.

**Key API for discovery**:
- `version_info.fixed()` → `dwFileVersion` (Major.Minor.Patch.Build)
- `version_info.translation()` → language codes, then `version_info.value(lang, "ProductName")` for name matching
- Must wrap in `spawn_blocking` (synchronous I/O)
- `fixed()` can return `None` — always fall back to string `"FileVersion"`

## GitHub Actions — Workflow Patterns

### Dynamic Matrix from workflow_dispatch

**Decision**: Two-job pattern (prepare → process) with `fromJson()`
**Rationale**: The prepare job scans manifests, outputs a JSON array of package IDs. The process job uses `strategy.matrix` with `fromJson()`. Max 256 jobs per workflow run (well above our needs).

```yaml
jobs:
  prepare:
    outputs:
      matrix: ${{ steps.scan.outputs.packages }}
    steps:
      - id: scan
        run: echo "packages=[...]" >> $GITHUB_OUTPUT
  test:
    needs: prepare
    strategy:
      matrix:
        package: ${{ fromJson(needs.prepare.outputs.matrix) }}
      max-parallel: 5
```

### Job Summary from Rust

**Decision**: Write directly to `GITHUB_STEP_SUMMARY` env var (file path)
**Rationale**: Simple `std::fs::OpenOptions::append()`. Supports markdown. Max 1MB per step. No external dependency needed.

### Cross-Repo PR via App Token

**Decision**: `actions/create-github-app-token@v3` with Nightwatch app credentials
**Rationale**: Already proven in `release.yml` (lines 27-32, 51-56). App has write access to both repos. Token used via `GH_TOKEN` env var for `gh` CLI.

### Always-Run Cleanup

**Decision**: `if: always()` + `continue-on-error: true` on cleanup steps
**Rationale**: Runs on success, failure, AND cancellation. `continue-on-error` prevents cleanup failures from failing the workflow.

## Catalog Schema — Current State

### Detection Table (Current v1)

Columns: `package_id`, `method`, `path`, `registry_key`, `registry_value`, `fallback_method`, `fallback_path`

**Missing for full DetectionConfig support**: `file_path` (separate from `path`), `version_regex`, `product_code`, `upgrade_code`, `inf_provider`, `device_class`, `inf_name`, recursive fallback chain.

### Schema Versioning

- `SUPPORTED_SCHEMA = "1"` constant in `reader.rs`
- Validated on every catalog open via `meta` table
- Per spec decision: stay on v1 (app not published), update table in-place
- Change `SUPPORTED_SCHEMA` check: no change needed if we modify the v1 schema in-place

### Migration Path (In-Place v1 Update)

Since the app isn't published, we update the detection table schema directly:

1. **Compiler** (manifests repo): update `schema.rs` to add new columns, serialize fallback as JSON blob in `fallback_config`
2. **Reader** (astro-up-core): update `detection_config()` SQL query to read new columns, deserialize `fallback_config` JSON
3. **Test fixture** (`create_fixture_catalog.rs`): update to match new schema
4. Drop legacy `fallback_method` + `fallback_path` columns, replace with `fallback_config` TEXT (JSON)

### Manifest TOML Deserialization

Confirmed: `Software` struct has `#[serde(default)]` on all optional fields. Raw TOML manifests deserialize directly via `toml::from_str::<Software>()`. No new struct needed — just a thin reader function.

## Registry Discovery Strategy

**Decision**: Enumerate all subkeys in 3 uninstall registry paths, match DisplayName against manifest `name` (primary) and package ID (fallback)
**Rationale**: Existing `registry.rs` searches by explicit registry key. Discovery needs blind enumeration. Match by product name is more reliable than slug-based matching.

**Search paths** (same as existing `find_uninstall_command`):
1. `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`
2. `HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall`
3. `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`

**Values to extract per match**: DisplayName, DisplayVersion, InstallLocation, UninstallString, QuietUninstallString, Publisher
