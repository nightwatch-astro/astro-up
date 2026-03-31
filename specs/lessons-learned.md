# Lessons Learned

## Wiring
- [005, 2026-03-30] Verify downloaded artifacts in memory before writing to disk — preserves previous valid state on failure

## Spec Quality
- [005, 2026-03-30] Check type compatibility with consumed specs during plan phase — slug Option<String> vs String mismatch caught only in STEP 14

## Process
- [005, 2026-03-30] Check crates.io feature names when upgrading major crate versions — reqwest 0.13 renamed rustls-tls to rustls
- [011, 2026-03-31] Windows crate API changes between minor versions — verify cfg(windows) code compiles on Windows CI before merging. windows 0.62 changed CreateProcessW to Option<PWSTR>, HANDLE is not Send, ShellExecuteW moved modules
- [011, 2026-03-31] Rust stable clippy evolves faster than expected — Windows CI may run newer Rust than local (1.94 vs 1.92). io_other_error lint only fires on newer clippy. Always use Error::other() instead of Error::new(ErrorKind::Other)
