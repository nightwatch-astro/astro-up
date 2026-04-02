# Data Model: Install Orchestration Engine

## New Types

### VersionFormat

Drives parser selection for version comparison. Stored in manifest `versioning.version_format` field, carried through catalog into the engine.

```
VersionFormat
├── Semver          # Default. Lenient parsing (v-prefix, 2-part, 4-part coercion)
├── Date            # YYYY.MM.DD or YYYY-MM-DD chronological comparison
└── Custom(String)  # Regex with capture groups, numeric group comparison
```

### PackageStatus

Result of comparing installed version against catalog version.

```
PackageStatus
├── UpToDate                    # installed == catalog latest
├── UpdateAvailable {           # installed < catalog latest (minor)
│     current: Version,
│     available: Version
│   }
├── MajorUpgradeAvailable {     # installed < catalog latest (major, semver only)
│     current: Version,
│     available: Version
│   }
├── NewerThanCatalog {          # installed > catalog latest (beta/dev)
│     current: Version,
│     catalog_latest: Version
│   }
├── NotInstalled                # Not detected on system
└── Unknown                     # Detection failed or no version info
```

### UpdatePlan

Ordered list of package updates with resolved dependencies.

```
UpdatePlan
├── items: Vec<PlannedUpdate>   # Topologically sorted
├── skipped: Vec<SkippedPackage># Filtered by policy or already up-to-date
└── warnings: Vec<String>       # Non-fatal issues (format mismatch, newer-than-catalog)

PlannedUpdate
├── package_id: PackageId
├── software: Software          # Full manifest data
├── current_version: Version
├── target_version: Version
├── version_entry: VersionEntry # Download URL, SHA256, etc.
├── version_format: VersionFormat
├── has_backup_config: bool     # Whether [backup] is defined for this package
└── dependencies: Vec<PackageId># Direct dependencies (for skip-on-failure)

SkippedPackage
├── package_id: PackageId
├── reason: SkipReason
└── status: PackageStatus
```

### SkipReason

Why a package was excluded from the update plan.

```
SkipReason
├── UpToDate
├── NewerThanCatalog
├── PolicyBlocked { policy: PolicyLevel }  # Major update blocked by minor-only policy
├── ManualOnly                              # Policy is Manual, no --allow-major
├── Disabled                                # Policy is None
└── DependencyFailed { dep_id: PackageId }  # Dependency failed during execution
```

### OperationRecord

Append-only history record for install/update/uninstall operations.

```
OperationRecord
├── id: i64                     # Auto-increment
├── package_id: String
├── operation_type: OperationType  # Install, Update, Uninstall
├── from_version: Option<String>
├── to_version: Option<String>
├── status: OperationStatus     # Success, Failed, Cancelled, RebootPending
├── duration_ms: u64
├── error_message: Option<String>
└── created_at: DateTime<Utc>

OperationType: Install | Update | Uninstall
OperationStatus: Success | Failed | Cancelled | RebootPending
```

### PipelineStep

Individual step result during single-package update execution.

```
PipelineStep
├── Compare    → PackageStatus
├── Download   → PathBuf (installer path)
├── Backup     → Option<BackupResult> (None if no backup config)
├── Install    → InstallResult
└── Verify     → PackageStatus (re-detect post-install)
```

## Extended Types (modifications to existing)

### Version (types/version.rs) — extended

Add:
- `compare_with_format(other: &Version, format: &VersionFormat) -> Ordering` — format-aware comparison
- `parse_date(raw: &str) -> Option<NaiveDate>` — YYYY.MM.DD or YYYY-MM-DD
- `parse_custom(raw: &str, regex: &Regex) -> Option<Vec<u64>>` — extract numeric capture groups
- `is_major_upgrade(from: &Version, to: &Version) -> bool` — true if major component differs (semver only)

### Events (events.rs) — extended

Add orchestration-level events:
- `PlanReady { total: usize, skipped: usize }` — plan computed
- `PackageStarted { package_id, step_count }` — starting single-package pipeline
- `PackageComplete { package_id, status }` — single-package done
- `PackageSkipped { package_id, reason }` — skipped due to policy or dependency failure
- `ProcessBlocking { package_id, process_name, pid }` — waiting for process to close
- `OrchestrationComplete { succeeded, failed, skipped }` — all done

## Relationships

```
Software ──has──> VersioningConfig ──contains──> version_format field
                                   ──contains──> UpdatePolicy

Catalog  ──resolves──> PackageSummary + VersionEntry

Detector ──detects──>  Version (installed)

UpdatePlanner:
  Catalog versions + Detected versions + UpdatePolicy → UpdatePlan
  DependencyConfig → topological order

UpdateOrchestrator:
  UpdatePlan → for each PlannedUpdate:
    Compare → Download → Backup → Install → Verify
    └── emits PipelineEvent at each step
    └── writes OperationRecord on completion

Lock file ──guards──> UpdateOrchestrator (one instance at a time)
Process check ──gates──> Backup→Install phase per package
```

## State Transitions

### Package Pipeline State

```
Pending → Comparing → Downloading → BackingUp → Installing → Verifying → Complete
                                                                        → Failed
    ↓ (at any point)
  Cancelled
    ↓ (on dependency failure)
  Skipped
```

### Operation Status

```
(started) → Success
          → Failed { error_message }
          → Cancelled
          → RebootPending
```
