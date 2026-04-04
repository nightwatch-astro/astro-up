# Data Model: 023-lifecycle-testing

## Entities

### DiscoveryCandidate

A detection signature found during blind probing.

| Field | Type | Description |
|-------|------|-------------|
| method | DetectionMethod | Which detection method found this |
| config | DetectionConfig | Generated config for this candidate |
| confidence | DiscoveryConfidence | High / Medium / Low |
| version | Option<Version> | Extracted version, if any |
| install_path | Option<String> | Discovered install location |
| display_name | Option<String> | Registry DisplayName that matched |
| registry_key_name | Option<String> | Registry subkey name (for config generation) |

### DiscoveryConfidence

Ordered priority for ranking candidates.

| Level | Criteria | Tie-break |
|-------|----------|-----------|
| High | Registry with version, PeFile with version | Prefer Registry |
| Medium | Registry without version, FileExists | Prefer Registry |
| Low | WMI, ASCOM, DriverStore | First match |

### DiscoveryResult

Output of a full discovery scan for one package.

| Field | Type | Description |
|-------|------|-------------|
| candidates | Vec<DiscoveryCandidate> | All found signatures, sorted by confidence |
| best_config | Option<DetectionConfig> | Generated config from best candidate (with fallback from second-best) |
| probed_locations | Vec<ProbedLocation> | All locations checked, for debugging |

### ProbedLocation

Debug info for a single probe attempt.

| Field | Type | Description |
|-------|------|-------------|
| method | DetectionMethod | Which method was tried |
| location | String | Registry path, file path, or WMI query |
| result | String | "found", "not_found", "error: ..." |

### LifecyclePhase

One phase of the lifecycle test.

| Variant | Fields |
|---------|--------|
| Download | url, filename, sha256, bytes |
| Install | method, exit_code, reboot_required |
| Detect | discovery_result |
| VerifyInstall | detection_result, expected_version |
| Uninstall | command, exit_code |
| VerifyRemoval | detection_result, leftover_files, leftover_registry_keys |
| Report | — (aggregates above) |

### LifecycleReport

Full test results for one package.

| Field | Type | Description |
|-------|------|-------------|
| package_id | String | Package tested |
| version | String | Version tested |
| phases | Vec<PhaseResult> | Per-phase results |
| discovered_config | Option<DetectionConfig> | Best detection config found |
| overall_status | LifecycleStatus | Pass / PartialPass / Fail |

### PhaseResult

Result of a single lifecycle phase.

| Field | Type | Description |
|-------|------|-------------|
| phase | String | Phase name |
| status | PhaseStatus | Pass / Fail / Skipped |
| duration_ms | u64 | Time taken |
| exit_code | Option<i32> | Process exit code, if applicable |
| logs | Vec<String> | Relevant log lines |
| warnings | Vec<String> | Non-fatal issues (leftovers, etc.) |

## Detection Table Schema (Updated v1)

Current:
```sql
CREATE TABLE detection (
    package_id TEXT PRIMARY KEY,
    method TEXT NOT NULL,
    path TEXT,
    registry_key TEXT,
    registry_value TEXT,
    fallback_method TEXT,
    fallback_path TEXT
);
```

Updated (in-place, still v1):
```sql
CREATE TABLE detection (
    package_id TEXT PRIMARY KEY,
    method TEXT NOT NULL,
    file_path TEXT,
    registry_key TEXT,
    registry_value TEXT,
    version_regex TEXT,
    product_code TEXT,
    upgrade_code TEXT,
    inf_provider TEXT,
    device_class TEXT,
    inf_name TEXT,
    fallback_config TEXT  -- JSON blob, max depth 3
);
```

Changes:
- `path` → `file_path` (renamed for clarity)
- Added: `version_regex`, `product_code`, `upgrade_code`, `inf_provider`, `device_class`, `inf_name`
- `fallback_method` + `fallback_path` → `fallback_config` (JSON blob of full DetectionConfig, max depth 2)

## Relationships

```
Software 1──1 DetectionConfig (from manifest TOML)
Software 1──1 InstallConfig (from manifest TOML)
Software 1──1 CheckverConfig (has autoupdate URL)

DiscoveryResult *──* DiscoveryCandidate (ranked list)
DiscoveryResult 1──1 DetectionConfig (best_config, generated)

LifecycleReport 1──* PhaseResult (7 phases max)
LifecycleReport 1──1 DetectionConfig (discovered_config)

LedgerEntry.install_path ← DetectionResult.install_path (populated after install)
```
