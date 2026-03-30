# Decisions Report: 006-registry-pe-detection

**Created**: 2026-03-29
**Mode**: Unattended, then interactive clarify with user

## Decisions Made

### D1: pelite for cross-platform PE detection
**Choice**: pelite crate for reading PE version headers.
**Reasoning**: Works on Linux/macOS CI without Windows. Reads PE headers from file, no execution needed.

### D2: Check both registry views unconditionally
**Choice**: Always check both native and WOW6432Node on 64-bit Windows.
**Reasoning**: Many astro apps (PHD2, SharpCap) are still 32-bit.

### D3: Version coercion via spec 003 Version::parse()
**Choice**: Reuse the existing lenient parser.
**Reasoning**: Consistent version handling across the entire application.

### D4: ASCOM Platform 7 minimum
**Choice**: Require ASCOM Platform 7+.
**Reasoning**: Current version, maintains profile registry. Older versions EOL.

## Clarify-Phase Decisions (Interactive)

### C1: Event-driven detection cache
**Decision**: In-memory cache of detection results. Invalidated per-package on install/update. Full invalidation on explicit `scan`. External changes caught on next scan.
**Reasoning**: Fast repeated lookups without staleness for our own operations.

### C2: Install ledger is a path resolver, not a detection method
**Decision**: Detection chain stays: registry → pe_file → file_exists. PE detection resolves paths from manifest config OR ledger. The ledger provides the path, PE provides the version.
**Reasoning**: For portable/astro-up-installed apps, we know the EXE path from the ledger. PE detection at that path catches self-updates too.

### C3: PE detection catches self-updates
**Decision**: PE always reads current file version, not ledger version. If an app self-updates (e.g., NINA checks its own updates), PE reports the actual installed version.
**Reasoning**: PE headers are ground truth.

### C4: Default chain with manifest override
**Decision**: Fixed default: registry → pe_file → file_exists. Manifest overrides for edge cases (ASCOM Profile, WMI, config_file). Most manifests don't need to override.

### C5: Alpaca dropped — ASCOM detection is registry-only
**Decision**: ASCOM Alpaca is a device control protocol, not a driver management tool. It tells you what's connected, not what driver version is installed. Spec 009 (Alpaca client) is dropped. ASCOM driver detection uses the Profile registry keys only.

## Questions I Would Have Asked

### Q1: Should we detect apps installed by other package managers?
**My decision**: No — we detect via registry and PE regardless of install source. If it has a registry entry or known EXE path, we find it.

### Q2: Should detection report the install method?
**My decision**: No — detection reports presence and version only. Install method comes from the manifest.
