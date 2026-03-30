# Data Model: 011 Installer Execution

**Date**: 2026-03-30

## New Types

### InstallRequest

Input to the installer service. Constructed by the orchestration layer (spec 012).

| Field | Type | Description |
|-------|------|-------------|
| package_id | String | Package identifier from catalog |
| package_name | String | Display name for events/logging |
| installer_path | PathBuf | Path to downloaded installer file |
| install_dir | Option\<PathBuf\> | Custom install directory override (None = type default) |
| install_config | InstallConfig | From manifest — method, switches, elevation, exit codes |
| timeout | Duration | Effective timeout (manifest override or default 600s) |
| quiet | bool | Silent mode (true for automated, false for interactive) |
| cancel_token | CancellationToken | For user cancellation |
| event_tx | broadcast::Sender\<Event\> | Event channel |

### InstallResult

Return type from installer. Success states only — failures are `CoreError`.

| Variant | Fields | Description |
|---------|--------|-------------|
| Success | path: Option\<PathBuf\> | Install completed. Path is install location if known. |
| SuccessRebootRequired | path: Option\<PathBuf\> | Installed but reboot needed to complete. |
| Cancelled | — | User cancelled during install. |

### UninstallRequest

Input for uninstall operations.

| Field | Type | Description |
|-------|------|-------------|
| package_id | String | Package identifier |
| uninstall_command | Option\<String\> | From registry (QuietUninstallString or UninstallString) |
| install_dir | Option\<PathBuf\> | For ZIP/portable deletion |
| method | InstallMethod | Determines uninstall strategy |
| quiet | bool | Silent uninstall |
| confirm | bool | Caller sets true after prompting user (required for ZIP/portable deletion) |
| cancel_token | CancellationToken | For cancellation |

## Modified Types (Cross-spec changes to spec 003)

### LedgerEntry (add field)

| Field | Type | Change |
|-------|------|--------|
| install_path | Option\<PathBuf\> | **NEW** — recorded after successful install. None for MSI/exe with unknown default location. |

### InstallConfig (add field)

| Field | Type | Change |
|-------|------|--------|
| timeout | Option\<Duration\> | **NEW** — per-manifest timeout override. Parsed with `humantime-serde`. Valid range: 10s–3600s. |

### Installer trait (signature change)

```rust
// Before (spec 003):
async fn install(&self, opts: &InstallOptions) -> Result<(), CoreError>;

// After (spec 011):
async fn install(&self, request: &InstallRequest) -> Result<InstallResult, CoreError>;
```

`InstallOptions` is replaced by `InstallRequest` which carries all context needed for execution.

## New Event Variants

| Variant | Fields | When |
|---------|--------|------|
| InstallFailed | id: String, error: String | Installer exited with non-success, non-reboot code |
| InstallRebootRequired | id: String | Installer exited with reboot-required code (3010) |

## Exit Code Resolution Flow

```
exit_code → success_codes check → known_exit_codes check → default interpretation

1. If exit_code == 0 → Success
2. If exit_code in success_codes → Success (semantic from known_exit_codes is informational)
3. If exit_code in known_exit_codes → map to KnownExitCode variant
4. If exit_code == 740 → ElevationRequired (universal Windows convention)
5. If exit_code == 3010 → RebootRequired (universal Windows convention)
6. Else → InstallerFailed with raw exit code
```

## State Transitions

```
InstallRequest received
  → [pre_install hooks] → abort on failure
  → [elevation check] → re-exec if needed
  → [spawn process] → with Job Object if bootstrapper
  → [wait with timeout] → kill on timeout
  → [interpret exit code]
    → Success → record ledger → emit InstallComplete
    → RebootRequired → record ledger → emit InstallRebootRequired
    → ElevationRequired → retry with elevation
    → Failed → emit InstallFailed
  → [post_install hooks] → warn on failure, don't abort
```
