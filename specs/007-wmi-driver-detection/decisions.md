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
