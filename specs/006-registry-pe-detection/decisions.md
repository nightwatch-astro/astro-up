# Decisions Report: 006-registry-pe-detection
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions
- **pelite for PE detection**: Cross-platform, works in Linux CI without Windows. No runtime dependency.
- **Check both WoW6432Node and native registry**: Many astro apps are 32-bit on 64-bit Windows.
- **Version string coercion**: Strip 4th component (3.1.2.1001 → 3.1.2), pad missing (3.1 → 3.1.0). Matches spec 003 Version type.
- **ASCOM Profile as separate detection method**: ASCOM uses non-standard registry paths. Warrants its own method rather than hacking generic registry detection.

## Questions I Would Have Asked
- Q1: Should detection cache results? Decision: No — detection is fast (<5s) and cached results would mask uninstalls. Cache at the engine level if needed.
- Q2: Support DriverStore detection here or separate spec? Decision: Separate (spec 007, WMI). Different API, different use case.

## Clarify-Phase Decisions

### C1: Check both registry views unconditionally
**Decision**: Always check both native and WOW6432Node. Don't try to guess bitness from the manifest — it's fragile and many apps change bitness between versions.

### C2: Version coercion matches spec 003 Version::parse()
**Decision**: Reuse the existing lenient parser from spec 003 (strip 4th component, pad missing, suffix as pre-release). No separate parsing logic in detection.

### C3: ASCOM Profile path includes space in "Drivers"
**Decision**: The standard ASCOM registry path is `ASCOM\Camera Drivers\ASCOM.Simulator.Camera`. The space is intentional and must be preserved in queries.

### C4: No detection caching — fast enough without it
**Decision**: Full scan of ~95 packages takes <5s. Caching adds staleness risk (uninstall not detected) for minimal perf gain. Orchestration engine can cache at a higher level if needed.
