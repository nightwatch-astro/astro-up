# Decisions Report: 007-wmi-driver-detection
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions
- **Separate spec from registry detection**: WMI is a different API with different failure modes. Keeping it separate makes each spec independently testable.
- **Wildcard VID:PID matching**: `03C3:*` matches all ZWO products. Vendors use a single VID with per-product PIDs.
- **No USB hotplug monitoring in this spec**: Detection is on-demand (scan), not event-driven. Hotplug monitoring belongs in the GUI spec.

## Questions I Would Have Asked
- Q1: Should we detect driver age (install date) to warn about stale drivers? Decision: No — version comparison against latest is sufficient.
- Q2: Should detection work without hardware connected? Decision: Yes — WMI shows installed drivers regardless of whether the device is plugged in.

## Clarify-Phase Decisions

### C1: Return all matching drivers, not just "best match"
**Decision**: The caller (orchestration engine) decides which match is relevant. Detection returns all matches. This avoids hiding information.

### C2: 10-second WMI timeout
**Decision**: WMI queries are usually fast (<1s) but can hang when the service is busy. 10s is generous. On timeout, return Unavailable with reason.

### C3: cfg(windows) gating, not separate crate
**Decision**: WMI code compiles on all platforms (for CI) but the actual WMI calls are behind `cfg(windows)`. Non-Windows returns Unavailable. Same pattern as spec 006 for registry.

### C4: No USB hotplug events in this spec
**Decision**: Detection is on-demand (user triggers scan). Real-time USB hotplug monitoring is a GUI feature (spec 016/017) that would trigger a re-scan.
