# Decisions Report: 015-cli-interface
**Created**: 2026-03-29
**Mode**: Unattended

## Decisions Made

### D1: ratatui for progress, not a full TUI app
**Choice**: Use ratatui only for progress bars and tables during long operations. Not a persistent TUI application.
**Reasoning**: A full TUI (like the Go version's Bubble Tea) is over-engineered for a tool that runs commands and exits. Progress bars during downloads/installs are the only interactive element needed.

### D2: Auto-detect TTY for output mode
**Choice**: If stdout is a TTY, use colors and TUI. If piped, use plain text. `--json` overrides both.
**Reasoning**: Unix convention. Enables `astro-up list | grep guiding` without escape codes in the output.

### D3: Standalone binary, no Tauri dependency
**Choice**: astro-up-cli links only against astro-up-core + clap + ratatui. No WebView, no Tauri.
**Reasoning**: CLI users expect a small, fast binary. Tauri's WebView is only for the GUI. CLI should work on Windows Server without a display.

### D4: self-update via GitHub Releases
**Choice**: Check for new releases, download, replace binary. No Windows installer for CLI-only updates.
**Reasoning**: CLI self-update is a binary swap. The GUI has its own update mechanism (tauri-plugin-updater, spec 016). CLI users expect `self-update` to work like cargo-install or rustup.

## Clarify-Phase Decisions

### C1: Exit codes follow Unix convention
**Decision**: 0 = success, 1 = error, 2 = user cancelled (Ctrl+C). Not Windows-specific ERRORLEVEL patterns. Scripts check `$?` or `%ERRORLEVEL%` the same way.

### C2: `--category` filter on list and scan
**Decision**: Both `list` and `scan` support `--category guiding` to filter output. Consistent across read operations.

### C3: `config` is a subcommand group, not a flag
**Decision**: `astro-up config init` and `astro-up config show` as subcommands, not `astro-up --config-init`. Cleaner CLI structure with clap's subcommand support.

### C4: No `install` subcommand — use `update` with implicit install
**Decision**: `astro-up update nina-app` handles both first install and update. If not installed, it installs. If installed, it updates. Separate `install` command is confusing — what's the difference from update for an end user?

## Questions I Would Have Asked

### Q1: Should `list` show all catalog entries or only installed ones?
**My decision**: Only installed. `scan --all` shows the full catalog. `list` is for "what do I have?" not "what's available?"
**Impact if wrong**: Medium — some users might expect `list` to show everything. Easy to change.

### Q2: Should self-update require elevation on Windows?
**My decision**: Yes, if the binary is in Program Files. No, if it's in a user directory. Self-update detects the binary location and requests elevation if needed.
