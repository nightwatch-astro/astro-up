# Decisions Report: 011-installer-execution
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions
- **Per-type default switches**: InnoSetup, MSI, NSIS each have well-known silent switches. Defaults reduce manifest verbosity. Overridable via manifest `[install.switches]`.
- **Semantic exit codes over raw codes**: Map known codes to enum variants. Unknown codes get a generic "failed" with the raw code. Better UX than cryptic numbers.
- **10-minute default timeout**: Most installers complete within minutes. 10 min is generous but prevents indefinite hangs from stuck installers.
- **Zip-slip guard mandatory**: Always check extracted paths for `..` traversal. No configuration to disable — it's a security requirement.
- **No auto-reboot**: Report reboot-required to user. Never auto-reboot — that would be destructive during an imaging session.

## Questions I Would Have Asked
- Q1: Should we support unattended uninstall for upgrades? Decision: Defer to `upgrade_behavior` field in manifest. "uninstall_previous" triggers uninstall before install. But the uninstall spec is implicit here, not a separate spec.
- Q2: Should pre/post install hooks run elevated? Decision: Pre-hooks run at the same elevation as the installer. If the installer needs admin, pre-hooks run as admin too.
