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

## Clarify-Phase Decisions

### C1: Replace, not merge, default switches
**Decision**: If a manifest specifies custom silent switches, they fully replace the defaults. Merging is ambiguous (what if a custom switch conflicts with a default?). The manifest author knows their installer best.

### C2: Process tree waiting for bootstrappers
**Decision**: Some installers (Burn bootstrappers) spawn a child process and exit immediately. We need to wait for the entire process tree, not just the initial PID. Use Windows Job Objects to track child processes.

### C3: Smart ZIP extraction (avoid double nesting)
**Decision**: If a ZIP contains a single top-level directory (e.g., `NINA-3.1.2/`), extract its contents to the target. If it contains multiple top-level items, extract as-is. This matches user expectations.

### C4: Pre-install failure aborts, post-install failure warns
**Decision**: Pre-install hooks are for prerequisites (e.g., stopping a service). If they fail, the install would likely fail too. Post-install hooks are for cleanup (e.g., creating shortcuts). Not critical.

### C5: DownloadOnly packages skip execution entirely
**Decision**: Some packages (ASCOM driver packages with manual install wizards) can't be silently installed. DownloadOnly marks them as "download and open containing folder." The user handles installation manually.
