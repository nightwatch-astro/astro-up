# Quickstart: Stability and Hardening Audit

## What This Changes

This audit hardens the existing codebase without adding features. After implementation:

1. **Backup restore** validates every file path — no more path traversal risk
2. **Application survives panics** — one failing command doesn't crash everything
3. **All inputs validated** — commands reject paths outside app directories
4. **Errors reported, not swallowed** — users see feedback, logs have context
5. **Code simplified** — large files decomposed, duplicates consolidated
6. **Full tracing** — every I/O operation has structured logging
7. **Frontend lifecycle safe** — no stale data from unmounted components

## Development Setup

No new tooling required. Existing `just` commands cover everything:

```sh
just setup    # Installs deps (adds parking_lot automatically)
just check    # Runs all quality checks (matches CI)
just test     # Rust + Vue tests
just dev      # Tauri dev server for manual testing
```

## New Dependency

- `parking_lot = "0.12"` added to `astro-up-gui/Cargo.toml`
- Replaces `std::sync::Mutex` — `.lock()` no longer returns `Result`

## Key New Module

- `crates/astro-up-core/src/validation.rs` — shared path validation utilities
  - `validate_zip_entry()` — safe ZIP extraction
  - `validate_within_allowlist()` — command input validation
  - `validate_backup_sources()` — backup creation validation

## Testing the Changes

```sh
# Run all tests (includes new path traversal tests)
just test

# Run only core crate tests
cargo test -p astro-up-core

# Run only GUI crate tests (requires Tauri system deps)
cargo test -p astro-up-gui

# Run frontend tests
cd frontend && pnpm test
```

## Verification Checklist

After implementation, verify:
- [ ] `cargo clippy -- -D warnings` passes (no unwrap in production code)
- [ ] Path traversal test with `../../` entries passes
- [ ] App survives a mock panic in a command handler
- [ ] Frontend invoke errors show toast notifications
- [ ] No source file exceeds 500 lines (excluding tests)
