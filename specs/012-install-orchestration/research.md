# Research: Install Orchestration Engine

## R1: Version Format Parsing

**Decision**: Extend existing `Version` type with a `VersionFormat` enum that drives parsing strategy.

**Rationale**: The existing `types/version.rs` already handles lenient semver (v-prefix, 4-part, 2-part). Adding date and custom regex support is incremental — no new crate needed. The `semver` crate (already a dependency) does NOT support lenient parsing natively, but the project's hand-rolled `try_parse_lenient()` already fills that gap.

**Alternatives considered**:
- `version-compare` crate (45M downloads) — no date format support, no regex
- `versions` crate (7.6M downloads) — no date format, no regex
- `lenient_semver` crate — unmaintained since 2021
- New `ParsedVersion` enum (from manifests repo) — already partially ported; extend rather than replace

**Implementation approach**:
```
VersionFormat::Semver → existing try_parse_lenient()
VersionFormat::Date → parse YYYY.MM.DD or YYYY-MM-DD via chrono::NaiveDate
VersionFormat::Custom(regex) → compile regex, extract capture groups, compare numerically
```

## R2: Process Detection (Running Software Check)

**Decision**: Use `sysinfo` crate (already a dependency from spec 005 catalog lockfile) to check if a process is running by name.

**Rationale**: `sysinfo` is already in the dependency tree, cross-platform, and provides `System::processes_by_name()`. No new dependency needed.

**Alternatives considered**:
- Windows `CreateToolhelp32Snapshot` API via `windows` crate — Windows-only, more code
- `tasklist` command via `std::process::Command` — fragile, parsing stdout
- `psutil` crate — Python-ish API, unnecessary dependency

**API**:
```rust
use sysinfo::System;
let mut sys = System::new();
sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
let procs: Vec<_> = sys.processes_by_name("NINA.exe".as_ref()).collect();
// Returns Process with pid(), name(), exe()
```

## R3: Global Lock File

**Decision**: Use `fd-lock` crate for cross-process file locking with RAII guard and PID tracking for stale lock detection.

**Rationale**: `fd-lock` (actively maintained, last updated 2024) provides `RwLock<File>` with RAII guards — lock is automatically released on drop, preventing accidental lock leaks during long-running orchestration. Spec 005 already implements PID-based lockfile logic for the catalog — reuse the same pattern for stale lock detection (write PID to file, check if process is alive on lock failure).

**Alternatives considered**:
- `fs4` — extension trait API (manual unlock required), stale (last updated 2021)
- `file-lock` — unmaintained
- Manual `O_EXCL` creation — no advisory locking, can't detect stale locks
- Reuse spec 005's catalog lockfile code — similar pattern but different scope (catalog vs orchestration)

**Implementation approach**: Write PID to lock file, use `fd_lock::RwLock::new(file).try_write()`. On lock failure, read PID from file, check if process is alive via `sysinfo`. If dead, remove stale lock and retry. Lock guard stored in orchestrator struct — auto-released on drop.

## R4: Dependency Resolution (Topological Sort)

**Decision**: Hand-roll topological sort using Kahn's algorithm. No graph crate needed.

**Rationale**: The dependency graph is small (≤50 packages), acyclic (circular deps are an error), and only needs topological ordering. Kahn's algorithm is ~30 lines, detects cycles, and has no dependencies. A graph library like `petgraph` would be overkill.

**Alternatives considered**:
- `petgraph` — full graph library, 500KB+ compile time addition for a simple topo sort
- `daggy` — unmaintained since 2020
- `toposort-scc` — unnecessary SCC detection

**Implementation**: Build adjacency list from `DependencyConfig.requires`, run Kahn's. If queue empties before all nodes visited → cycle detected → abort with cycle path.

## R5: Disk Space Check

**Decision**: Use `sysinfo::Disks` for disk space query.

**Rationale**: `sysinfo` is already a dependency (spec 005, process detection). `Disks::new_with_refreshed_list()` provides available space per mount point. Match the download/install target path to the correct disk. No additional dependency needed.

**Alternative**: `fs4::available_space()` — simpler API but would add a dependency just for disk space now that locking uses `fd-lock`.

## R6: Operation History Storage

**Decision**: Add `operations` table to existing SQLite database. Use rusqlite (already a dependency in the catalog module).

**Rationale**: Single database file, single connection pool. The catalog module already establishes the SQLite pattern. Operations table is append-only with simple INSERT + SELECT queries.

**Schema**:
```sql
CREATE TABLE IF NOT EXISTS operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id TEXT NOT NULL,
    operation_type TEXT NOT NULL CHECK(operation_type IN ('install', 'update', 'uninstall')),
    from_version TEXT,
    to_version TEXT,
    status TEXT NOT NULL CHECK(status IN ('success', 'failed', 'cancelled', 'reboot_pending')),
    duration_ms INTEGER NOT NULL,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_operations_package ON operations(package_id);
CREATE INDEX IF NOT EXISTS idx_operations_created ON operations(created_at);
```

## Dependencies Summary

| Crate | Version | New? | Purpose |
|-------|---------|------|---------|
| semver | workspace | No | Semver parsing (existing) |
| chrono | workspace | No | Date version parsing (existing) |
| sysinfo | 0.38 | No | Process detection + stale lock check (existing from 005) |
| regex | 1 | Yes | Custom version format parsing |
| fd-lock | 4 | Yes | Global lock file with RAII guards |
| rusqlite | workspace | No | Operations table (existing from 005) |
| tokio-util | 0.7 | No | CancellationToken (existing from 010/011) |
