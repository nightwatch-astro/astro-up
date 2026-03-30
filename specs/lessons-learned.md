# Lessons Learned

## Wiring
- [005, 2026-03-30] Verify downloaded artifacts in memory before writing to disk — preserves previous valid state on failure

## Spec Quality
- [005, 2026-03-30] Check type compatibility with consumed specs during plan phase — slug Option<String> vs String mismatch caught only in STEP 14

## Process
- [005, 2026-03-30] Check crates.io feature names when upgrading major crate versions — reqwest 0.13 renamed rustls-tls to rustls
