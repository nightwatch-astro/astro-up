# Architecture Decision: Rust + Tauri Rewrite

> Date: 2026-03-28
> Decision type: Architecture decision + Technology choice + Feature proposal
> Knowledge source: Research with 12+ subagents (full codebase context)
> Confidence: Medium-High (60-70%)

## Executive Summary

After extensive research across architecture, market validation, maintenance costs, manifest pipelines, distribution models, GUI frameworks, language comparison, and frontend frameworks, the decision is to **rewrite astro-up in Rust with Tauri v2 for the GUI and Vue 3 + PrimeVue for the frontend**. The current Go + Wails v2 + Svelte stack works but has significant friction points (Wails binding complexity, WebView2 testing, Svelte ecosystem thinness). The project is early enough that a rewrite is viable, the manifest pipeline complexity is in the data model rather than the language, and Rust + Tauri provides a better long-term foundation for Windows system programming.

## Context

astro-up is a Windows application managing ~95 astrophotography software packages (29 EXE, 27 ZIP, 9 InnoSetup, 30 drivers/manual). Current stack: Go 1.26+ backend, Wails v2 GUI, Svelte 5 + shadcn-svelte frontend, urfave/cli + Bubble Tea CLI/TUI.

The manifest pipeline (separate repo, `astro-up/astro-up-manifests`) compiles 110 TOML manifests, checks vendor versions every 6 hours using 8 strategies, and produces signed JSON consumed by the client.

## Problem Validation

### The Pain is Real

- Astrophotographers manage 8-20 packages with complex interdependencies
- **67 of 95 manifests are drivers/firmware with zero auto-update capability**
- Vendors don't send update notifications — stale software causes mysterious failures ("plate solving broken" → ASTAP was a year out of date)
- **Zero competing tools exist** — extensive search across GitHub, Cloudy Nights, Reddit, ASCOM forums
- Target audience: 50K-150K Windows astrophotographers, strongly GUI-oriented
- The ASIAir ($200-300 appliance) proves willingness to pay to avoid software management

### Both Setup and Update Pain are Real

- **Setup pain is visible** — users post about it on forums ("so much different software to use")
- **Update pain is invisible** — users don't know they don't know about updates. They post symptoms, not causes.
- The invisible update problem is the stronger case for automated version checking

### Market Context (Open Source)

Market size and monetization concerns are irrelevant for an OSS project. The tool needs to be useful for the developer and a handful of others. "Zero competitors" = opportunity, not "no market."

## Research Findings

### 1. Build vs Buy: No Alternative Exists

- **Zero** astrophotography software managers exist on any platform
- Only 2 of 95 packages are in winget (Stellarium, NINA). Zero in Chocolatey.
- 67 of 95 are drivers/firmware that no package manager handles
- General-purpose PMs (Scoop, winget, Chocolatey) cover ~2% of the catalog
- The curated catalog pattern is proven (Homebrew, conda, vcpkg) but none covers this niche

### 2. Existing Package Managers Cannot Serve This Domain

| Manager | Brownfield | Custom Catalog | Driver Support | API |
|---------|-----------|---------------|----------------|-----|
| Scoop | No (by design) | Easy (buckets) | No | No (shell out) |
| winget | Yes (ARP registry) | Complex (REST source) | No | COM (no Go/Rust bindings) |
| Chocolatey OSS | No | NuGet feed | No | No |
| Chocolatey Biz | Yes (`choco sync`) | NuGet feed | No | $20k/yr |

**Scoop cannot adopt existing installs** — maintainer explicitly closed the request ("by design").
**winget has brownfield** but custom source requires REST API infrastructure and admin-required source registration.
**No PM handles drivers** — this is custom code regardless of backend choice.

### 3. Manifest Pipeline: Sound Architecture, Simplifiable

Compared Scoop's `checkver`/`autoupdate`, winget-pkgs automation (komac, wingetcreate), and Chocolatey AU:

- Scoop's declarative `checkver` maps almost 1:1 to astro-up's `[remote]` section
- astro-up's Go checker is superior to Scoop's PowerShell (cross-platform, typed, testable)
- **Simplification: rename `[remote]` to `[checkver]`, adopt Scoop variable convention**
- **Collapse CI matrix to single `CheckAll()` job** — 110 checks take <5min sequentially
- **Add SHA256 hashes to versions.json** — Scoop and winget both pin hashes
- **Add `meta.json`** (~100 bytes) for lightweight client change detection

### 4. Distribution: Current Approach is Right

- Raw GitHub + ETag + three-tier cache is well-designed
- Even at 10K DAU, bandwidth is ~7.2 GB/month with ETag caching
- OCI/ORAS, delta updates, WebSocket push — all overkill at this scale
- Only improvement worth making: `meta.json` for lightweight polling

### 5. Maintenance Cost of Current Codebase

| Metric | Value |
|--------|-------|
| Install/detect plumbing | 1,456 lines (27% of internal/) |
| Specs for plumbing | 11 of 25 (44%) |
| Integration test lines for install/detect | 48% of total |
| Windows CI jobs for install/detect | 5 of 7 |
| Vendors already dropped | 7 (6.4% attrition pre-launch) |
| Code removable if delegated | ~4,878 lines (source + tests) |

### 6. ASCOM Alpaca Makes COM Irrelevant

ASCOM's official position: "We strongly encourage you to write for ASCOM Alpaca." Alpaca is HTTP/JSON. The ASCOM Platform bridges COM↔Alpaca transparently. The `ascom-alpaca-rs` Rust crate exists (23 stars, 654 commits). A new `ascom-alpaca-core` crate was published on the day of this research.

### 7. Driver Detection is the Real Gap

None of the package managers handle driver detection. The expansion needed:
- `Win32_PnPSignedDriver` (WMI) — `wmi` Rust crate, 2.7M downloads, production-grade
- SetupAPI (`SetupDiGetClassDevs`) — via `windows-rs`
- ASCOM Profile registry — standard registry reads
- Deferred issue: #349

## Language Decision: Rust

### Scorecard

| Requirement | Rust | C# | Go (current) |
|-------------|------|-----|-------------|
| Registry | `winreg` (149M dl) | Built-in | `x/sys/windows` |
| PE Version | `pelite` (cross-platform!) | Built-in | Windows-only |
| WMI | `wmi` crate (typed serde) | Built-in | Not available |
| COM/ASCOM | IDispatch possible, Alpaca preferred | Native COM | Not practical |
| GUI | Tauri (104K stars) | WPF/WinUI (best native) | Wails (27K stars) |
| TOML | Gold standard (Cargo uses it) | Tomlyn (adequate) | go-toml |
| Minisign | First-party (by creator) | Nothing viable | go-minisign |
| Binary size | 2-5 MB | 60 MB (self-contained) | 15-20 MB |
| Runtime dep | None | .NET runtime | None |
| Compile time | 5-15min cold, 5-30s incremental | Fast | Fast |
| Cross-platform | `cfg(target_os)` clean separation | WPF is Windows-only | Build tags |

### Why Rust over Go

1. **Tauri > Wails** — more mature (104K vs 27K stars), better plugin ecosystem, better testing, framework-agnostic frontend
2. **Minisign is first-party** — no C# library exists at all
3. **PE parsing works cross-platform** — `pelite` parses PE files on Linux CI runners
4. **WMI is typed** — `wmi` crate deserializes to structs via serde (better than C#'s weakly-typed dictionary)
5. **No runtime dependency** — 2-5MB binary, no .NET installation
6. **`cfg(target_os)`** — clean platform separation for potential Linux/INDI later
7. ASCOM Alpaca eliminates C#'s biggest advantage (native COM)

### Why Rust over C#

- C#/WPF would be the best choice if starting fresh with no existing frontend code
- But: no minisign library in .NET, 60MB self-contained binary, and Vue + PrimeVue works excellently in Tauri
- The developer is familiar with Rust (completed nightwatch-esp32 C++ → Rust migration)

## GUI Framework Decision: Tauri v2

### Why Tauri over Wails

| Aspect | Tauri v2 | Wails v2 |
|--------|----------|----------|
| Stars | 104K | 27K |
| Binary size | 2-5 MB | 15-20 MB |
| Frontend freedom | Fully agnostic | Framework-agnostic but opinionated tooling |
| Plugin ecosystem | 30+ official plugins | Minimal |
| Auto-updater | Built-in (Ed25519) | Need go-selfupdate |
| System tray | Built-in plugin | Manual implementation |
| Testing | WebDriver + IPC mocks | CDP hack (broken on CI) |
| Type generation | TauRPC (community) | Built-in (buggy codegen) |
| Binding DX | `invoke('cmd', {args})` — clean | `window.go.gui.App.Method()` — convoluted path |

### Tauri Plugins for astro-up

**Must-have (replaces custom code):**
- `tauri-plugin-shell` — run installers
- `tauri-plugin-updater` — self-update (replaces go-selfupdate)
- `tauri-plugin-notification` — update notifications
- `tauri-plugin-store` — persistent config (replaces koanf file)
- `tauri-plugin-fs` — file access
- `tauri-plugin-http` — manifest/version fetching
- `tauri-plugin-global-shortcut` — keyboard shortcuts
- `tauri-plugin-single-instance` — prevent multiple instances
- `tauri-plugin-autostart` — start at login
- `tauri-plugin-window-state` — remember window size/position

**Nice-to-have:**
- `tauri-plugin-dialog` — confirmation dialogs, folder picker
- `tauri-plugin-log` — structured logging
- `tauri-plugin-os` — Windows version detection
- `tauri-plugin-process` — restart after update
- `tauri-plugin-opener` — open vendor download pages
- `tauri-plugin-cli` — CLI argument parsing

## Frontend Decision: Vue 3 + PrimeVue + VueQuery

### Why Vue 3 + PrimeVue

PrimeVue's DataTable is the killer feature — the main view IS a data table (software list with sort, filter, group by category, status badges, actions).

| Feature | Vue + PrimeVue | React + Mantine | React + shadcn/ui |
|---------|---------------|-----------------|-------------------|
| DataTable | Built-in (sort/filter/paginate/group/virtual scroll) | Basic table only, need TanStack Table | Need TanStack Table |
| Components | 80+ | 100+ (simpler primitives) | ~50 (copy-paste) |
| Dark mode | Built-in themes (Aura, Lara, Nora) | Built-in | Via CSS variables |
| Forms | Built-in components | @mantine/form | React Hook Form |
| Styling | Own design tokens, no Tailwind needed | Own CSS system | Tailwind required |
| Learning curve | Gentle | Moderate (hooks) | Moderate (hooks) |

### Server State: VueQuery (TanStack Query for Vue)

Wraps Tauri `invoke()` calls with caching, loading states, background refetch, cache invalidation. Same library as React Query, Vue adapter.

### No Need For

- Router (3-4 views, `ref('dashboard')` is enough)
- Client state management (Pinia/Vuex) — backend IS the state, VueQuery bridges it
- Form library — Settings page is ~15 fields, `v-model` is enough
- TanStack Table — PrimeVue DataTable covers this

## UniGetUI: Patterns to Adopt

From the 22K-star C# package manager GUI:

1. **ManagerCapabilities struct** — per-tool capability flags, UI toggles controls dynamically
2. **Operation queue with auto-retry** — on elevation-needed error, auto-retry with admin
3. **Three-tier elevation** — proactive (check manifest), reactive (retry on failure), user-controlled (checkbox)
4. **Ignored updates database** — per-package version ignoring with `*` wildcard
5. **Parallel manager init with timeout** — 60s timeout + `AttemptFastRepair()` fallback

**Mistakes to avoid:**
- CLI text parsing (fragile — use structured data)
- Static singletons (use dependency injection)
- No cross-manager dedup

## Installer Robustness (Deferred: #348)

Exit codes verified from official sources:

**NSIS**: 0 (success), 1 (user abort), 2 (script abort). Source: NSIS docs.
**InnoSetup**: 0-8 range. Key: 2 (cancel before install), 5 (cancel during install), 7/8 (PrepareToInstall failed). Source: issrc/Setup.MainFunc.pas.
**MSI**: 0, 1602 (user cancel), 1603 (fatal), 1618 (busy), 1638 (version conflict), 3010 (reboot needed). Source: Microsoft docs.
**Windows**: 5 (access denied), 740 (elevation required), 1223 (UAC denied). Source: WinError.h.

Product should map these to structured error types with user-facing messages.

## Deferred Issues

- **#347** — Local download ledger for non-detectable packages (may be superseded by #349)
- **#348** — Installer robustness: exit codes, timeouts, elevation, InstallDir, WebView2
- **#349** — Driver detection via WMI/SetupAPI/ASCOM Profile
- **#331** — Wails v3 migration (superseded by Tauri rewrite decision)

## Build & Distribution Decision

### Drop Portable Build

Tauri has built-in NSIS and WiX installer bundlers. The current goreleaser setup produces both a portable binary and an Inno Setup installer. With Tauri:

- **Tauri NSIS bundler** replaces both goreleaser + Inno Setup
- The portable build (bare .exe) is dropped — Tauri's installer handles everything
- Self-update via `tauri-plugin-updater` replaces `go-selfupdate`
- Scoop bucket continues to work (points to the installer or extracted binary)

### CLI as Separate Binary (Shared Core Crate)

The CLI and GUI are **separate binaries** sharing a core library crate:

```
astro-up/
  crates/
    astro-core/       # Shared: detect, install, download, engine, catalog, config
    astro-cli/        # clap + ratatui, depends on astro-core. No Tauri/WebView dep.
    astro-gui/        # Tauri + Vue + PrimeVue, depends on astro-core
```

**Why separate, not bolted onto Tauri:**
- CLI has zero GUI dependencies — works on headless servers, SSH, CI pipelines
- CLI binary is tiny (~2MB), no WebView2 needed
- Can be distributed independently via Scoop, cargo install, or as a standalone download
- Tauri's `tauri-plugin-cli` only does argument parsing — it still opens a window context
- The `clap` crate is the de facto Rust CLI framework (700M+ downloads) — first-class CLI experience
- `ratatui` for TUI progress (replaces Bubble Tea) — also no GUI dependency

**Both binaries share `astro-core`** which contains all business logic: manifest parsing, detection, download, install orchestration, version checking, ASCOM Alpaca client, config management. Each frontend (CLI and GUI) is a thin wrapper.

The Tauri installer bundles both binaries. The CLI is also available standalone.

## Migration Plan

### Phase 1: Manifest Pipeline Simplification (no rewrite, current Go codebase)

- Rename `[remote]` → `[checkver]`, adopt Scoop `$version` variable convention
- Collapse CI matrix to single `CheckAll()` job
- Add SHA256 hashes to versions.json
- Add `meta.json` for lightweight client change detection
- Default installer switches per type (eliminate repeated `quiet_args`)

### Phase 2: Rust + Tauri Rewrite

**Backend (Rust):**

| Go Package | Rust Equivalent | Crate |
|-----------|-----------------|-------|
| internal/config | Config loading | `toml` + `figment` or custom |
| internal/core | Domain types | Custom structs + traits |
| internal/detect | Registry + PE + WMI | `winreg` + `pelite` + `wmi` |
| internal/download | HTTP + hash | `reqwest` + `sha2` |
| internal/install | Installer execution | `tauri-plugin-shell` |
| internal/engine | Orchestration | Custom (adopt UniGetUI patterns) |
| internal/catalog | Manifest catalog | `toml` + `serde` |
| internal/provider | GitHub/GitLab API | `octocrab` + `reqwest` |
| internal/scrape | HTML scraping | `scraper` crate |
| internal/selfupdate | Self-update | `tauri-plugin-updater` |
| internal/backup | Config backup | `tauri-plugin-fs` |
| internal/logging | Logging | `tauri-plugin-log` |
| go-minisign | Signature verification | `minisign-verify` |
| koanf | Layered config | `tauri-plugin-store` |
| urfave/cli | CLI | `clap` |
| bubbletea | TUI | `ratatui` |

**Frontend (Vue 3 + PrimeVue):**

```
src/
  App.vue                    # Root with sidebar nav
  views/
    Dashboard.vue            # PrimeVue DataTable, status badges
    Settings.vue             # PrimeVue form components
    CustomTools.vue           # Add/remove custom tools
  composables/
    useSoftware.ts           # VueQuery: invoke('list_software')
    useConfig.ts             # VueQuery: invoke('get_config')
    useUpdates.ts            # VueQuery: invoke('check_updates')
  components/
    StatusBadge.vue          # PrimeVue Badge
    DownloadProgress.vue     # PrimeVue ProgressBar
```

**Tauri plugins (16):**
shell, updater, notification, store, fs, http, global-shortcut, single-instance, autostart, window-state, dialog, log, os, process, opener, cli

### Phase 3: Feature Expansion

- Driver detection via WMI (#349)
- Installer robustness: exit codes, timeouts, elevation (#348)
- Setup wizard mode ("what telescope do you have?" flow)
- ASCOM Alpaca integration for driver queries

## Risk Assessment

### Red Flags That Should Trigger Reconsideration

1. Rust compile times significantly impacting development velocity
2. `wmi` or `winreg` crate has breaking changes without migration path
3. Tauri v2 has critical Windows-specific bugs not fixed within reasonable timeframe
4. Vue 3 or PrimeVue enters maintenance mode
5. ASCOM Alpaca adoption stalls, COM interop becomes necessary

### Mitigations

- **Compile times**: `cargo watch` for incremental, aggressive CI caching, `sccache`
- **Crate stability**: `winreg` (149M downloads) and `wmi` (2.7M) are production-proven
- **Tauri stability**: 104K stars, backed by CrabNebula (commercial entity), security-audited
- **Vue/PrimeVue**: Vue is established (10+ years), PrimeVue backed by PrimeTek (company)
- **ASCOM**: COM interop via `windows-rs` IDispatch is available as fallback

## Conclusion

The rewrite to Rust + Tauri + Vue + PrimeVue is justified because:

1. The project is early enough (pre-launch, majority of complexity in pipeline)
2. Tauri's plugin ecosystem replaces ~1,371 lines of custom Go code
3. Testing story is materially better (WebDriver vs CDP hack)
4. Binary size drops from 15-20MB to 2-5MB
5. The developer has Rust experience and finds Wails convoluted
6. PrimeVue's DataTable eliminates the need for TanStack Table
7. ASCOM Alpaca makes the COM interop advantage of C# irrelevant
8. `pelite` enables PE version parsing in Linux CI (cross-platform testing)
9. The manifest pipeline (TOML + Go checker + signed JSON) is the proven core — it stays as-is initially, can be ported to Rust later
