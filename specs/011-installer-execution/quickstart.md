# Quickstart: 011 Installer Execution

## What this feature does

Executes downloaded installers silently on Windows — handling 10 installer types, exit code interpretation, admin elevation, ZIP extraction with security guards, process tree management, and install ledger recording.

## Key files

| File | Purpose |
|------|---------|
| `crates/astro-up-core/src/install/mod.rs` | InstallerService facade, Installer trait impl |
| `crates/astro-up-core/src/install/types.rs` | InstallRequest, InstallResult, ExitCodeOutcome |
| `crates/astro-up-core/src/install/switches.rs` | Default silent switches per InstallMethod |
| `crates/astro-up-core/src/install/exit_codes.rs` | Exit code interpretation logic |
| `crates/astro-up-core/src/install/zip.rs` | ZIP extraction + zip-slip guard |
| `crates/astro-up-core/src/install/hooks.rs` | Pre/post hook execution |
| `crates/astro-up-core/src/install/elevation.rs` | `cfg(windows)` — admin check, sudo/runas |
| `crates/astro-up-core/src/install/process.rs` | `cfg(windows)` — spawn, Job Objects, timeout |
| `crates/astro-up-core/src/install/uninstall.rs` | `cfg(windows)` — registry lookup, removal |
| `crates/astro-up-core/src/install/ledger.rs` | Record LedgerEntry after install |
| `crates/astro-up-core/src/traits.rs` | Modified Installer trait signature |
| `crates/astro-up-core/src/events.rs` | New InstallFailed, InstallRebootRequired variants |
| `crates/astro-up-core/src/types/install.rs` | Add timeout field to InstallConfig |
| `crates/astro-up-core/src/ledger.rs` | Add install_path field to LedgerEntry |

## How to run

```sh
just check    # Clippy + fmt + tests (cross-platform)
just test     # Full test suite
```

## Cross-platform notes

- `switches.rs`, `exit_codes.rs`, `zip.rs`, `hooks.rs`, `types.rs`, `ledger.rs` — compile and test everywhere
- `elevation.rs`, `process.rs`, `uninstall.rs` — `#[cfg(windows)]` only
- `InstallerService` trait impl uses `#[cfg(windows)]` for real execution, `#[cfg(not(windows))]` returns `CoreError::Io(Unsupported)`

## Dependencies added

- `zip = "2"` — ZIP extraction
- `windows = "0.62"` — Job Objects, elevation, process management (`cfg(windows)` only)
- tokio `"process"` feature — async process spawning
