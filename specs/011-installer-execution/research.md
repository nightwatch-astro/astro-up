# Research: 011 Installer Execution

**Date**: 2026-03-30

## Dependency Research

### zip crate (v2.x)

**Decision**: Use `zip = "2"` for ZIP extraction
**Rationale**: De facto standard for ZIP in Rust. Appears on blessed.rs and lib.rs. Provides `enclosed_name()` for built-in zip-slip protection (CVE-2024-30407 fix). Synchronous API — wrap in `spawn_blocking`.
**Alternatives considered**: `async-zip` (immature), manual `flate2` + directory walking (unnecessary complexity)

Key API:
- `ZipArchive::new(reader)` — requires `Read + Seek`
- `entry.enclosed_name()` — returns `None` for zip-slip paths (`..`, absolute). **Must use this, never raw `entry.name()`**
- No built-in `extract_all()` — iterate entries manually
- Single-root detection: scan all entry prefixes manually, strip common root

### windows crate (v0.62)

**Decision**: Use `windows = "0.62"` with targeted feature flags
**Rationale**: Official Microsoft crate, on blessed.rs. Lockfile already has 0.62.2 transitively. Provides safe(r) wrappers over raw FFI. Preferred over `windows-sys` for new code.
**Alternatives considered**: `windows-sys` (raw FFI, no abstractions), `runas` crate (unmaintained wrapper around single API call)

Feature flags needed:
- `Win32_System_JobObjects` — `CreateJobObjectW`, `AssignProcessToJobObject`, `SetInformationJobObject`
- `Win32_System_Threading` — `CreateProcessW`, `CREATE_SUSPENDED`, `ResumeThread`, `WaitForSingleObject`
- `Win32_UI_Shell` — `ShellExecuteExW` (elevation), `IsUserAnAdmin`
- `Win32_Security` — token membership checks

Key gotcha: `ShellExecuteExW` with `runas` returns a process handle that **cannot be assigned to a Job Object** from the non-elevated context (different security context). Elevated + Job Object requires the elevated process itself to create the job.

### tokio process feature

**Decision**: Add `"process"` feature to existing tokio dependency
**Rationale**: Already using tokio 1.x. `tokio::process::Command` provides async process spawning with `kill_on_drop`, composable with `timeout` and `CancellationToken` via `select!`.
**Alternatives considered**: `std::process::Command` in `spawn_blocking` (loses async composability)

Key gotcha: `child.kill()` only kills direct process, NOT process tree. Job Objects required for bootstrapper-style installers.

## Execution Strategy

**Two-tier approach**:

| Tier | When | How |
|------|------|-----|
| Simple | Most installers (InnoSetup, MSI, NSIS, WiX, exe) | `tokio::process::Command` + timeout + CancellationToken |
| Job Object | Bootstrappers (Burn, spawning installers) | `CreateProcessW` + `CREATE_SUSPENDED` + Job Object + `spawn_blocking(WaitForSingleObject)` |
| Elevation | When admin required | `ShellExecuteExW runas` or `sudo.exe` (detected at runtime) |
| ZIP | ZIP/ZipWrap/Portable | `zip::ZipArchive` in `spawn_blocking` + `enclosed_name()` |

## Cargo.toml Changes

```toml
# Add to [dependencies]
zip = "2"

# Add to [target.'cfg(windows)'.dependencies]
windows = { version = "0.62", features = [
    "Win32_System_JobObjects",
    "Win32_System_Threading",
    "Win32_UI_Shell",
    "Win32_Security",
] }

# Update existing tokio to add "process"
tokio = { version = "1", features = ["fs", "time", "sync", "process"] }
```
