# Migration Plan: astro-up → Rust + Tauri + Vue + PrimeVue

> Date: 2026-03-28
> Organization: nightwatch-astro
> New repository: nightwatch-astro/astro-up
> Reference: [Architecture Decision](debate-architecture-rewrite-rust-tauri.md)

## Overview

Fresh start under the `nightwatch-astro` organization. All repositories migrate from the `astro-up` org. The Go codebase remains functional until the Rust version reaches feature parity, then old repos are archived.

## Organization Migration

| Old (astro-up org) | New (nightwatch-astro org) | Action |
|---|---|---|
| `astro-up/astro-up` | `nightwatch-astro/astro-up` | Fresh Rust repo (not a fork) |
| `astro-up/astro-up-manifests` | `nightwatch-astro/astro-up-manifests` | Fork/transfer + simplify CI |
| `astro-up/astro-up.github.io` | `nightwatch-astro/astro-up-docs` | Rebuild with Starlight, custom domain |
| `astro-up/scoop-bucket` | `nightwatch-astro/scoop-bucket` | Fork/transfer, update for new binary |

**After feature parity:** archive all `astro-up/` repos as read-only with a notice pointing to `nightwatch-astro/`.

**Domain:** Reuse existing custom domain for docs site. GitHub Pages source: workflow deploy.

## Repository Structure

```
nightwatch-astro/astro-up/                    # Main app repo
├── .github/
│   └── workflows/                            # CI: rust, vue, integration, release
├── .specify/                                 # Speckit workspace
├── crates/
│   ├── astro-up-core/                           # Shared library crate
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/                        # → future astro-up-types crate
│   │       ├── detect/                       # → future astro-up-detect crate
│   │       │   ├── mod.rs
│   │       │   ├── registry.rs               # cfg(windows)
│   │       │   ├── pefile.rs                 # cross-platform via pelite
│   │       │   ├── wmi.rs                    # cfg(windows)
│   │       │   └── composite.rs
│   │       ├── download/                     # → future astro-up-download crate
│   │       ├── install/                      # → future astro-up-install crate
│   │       ├── engine/                       # → future astro-up-engine crate
│   │       ├── catalog/                      # → future astro-up-catalog crate
│   │       ├── providers/                    # → future astro-up-providers crate
│   │       ├── config/
│   │       └── backup/
│   ├── astro-up-cli/                            # CLI binary: clap + ratatui
│   │   └── src/
│   │       ├── lib.rs                        # CLI logic (testable without subprocess)
│   │       └── main.rs                       # Entry point: parse args, dispatch to lib
│   └── astro-up-gui/                            # Tauri app binary
│       ├── src/
│       │   ├── lib.rs                        # Tauri commands (testable without window)
│       │   ├── main.rs                       # Tauri app init, window creation
│       │   ├── commands.rs                   # Invoke handlers
│       │   └── tray.rs                       # System tray
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       └── build.rs
├── frontend/                                 # Vue 3 + PrimeVue + VueQuery
│   ├── src/
│   │   ├── App.vue
│   │   ├── views/
│   │   │   ├── Dashboard.vue                 # PrimeVue DataTable
│   │   │   ├── Settings.vue                  # PrimeVue form components
│   │   │   └── CustomTools.vue
│   │   ├── composables/
│   │   │   ├── useSoftware.ts                # VueQuery + invoke
│   │   │   ├── useConfig.ts
│   │   │   └── useUpdates.ts
│   │   └── components/
│   │       ├── StatusBadge.vue
│   │       └── DownloadProgress.vue
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
├── research/                                 # Architecture decisions (copied from old repo)
│   ├── debate-architecture-rewrite-rust-tauri.md
│   └── migration-plan-rust-tauri.md
├── specs/                                    # Speckit specs
├── Cargo.toml                                # Workspace root
├── Cargo.lock
├── CLAUDE.md
├── LICENSE                                   # Apache-2.0
├── cliff.toml                                # git-cliff for changelogs
└── README.md

nightwatch-astro/astro-up-manifests/          # Manifest repo (migrated)
├── manifests/{category}/{vendor}-{product}.toml
├── compiler/                                 # TOML → JSON compiler
├── checker/                                  # Version checker (Go or Rust)
├── .github/workflows/
│   ├── compile-manifests.yml
│   └── check-versions.yml                    # Simplified: single CheckAll job
├── manifests.json                            # Compiled + signed
├── versions.json                             # Versions + SHA256 + signed
├── meta.json                                 # Lightweight change detection
└── *.minisig                                 # Minisign signatures

nightwatch-astro/astro-up-docs/               # Documentation site
├── src/content/docs/                         # Starlight content
├── src/components/
│   └── SoftwareCatalog.astro                 # Dynamic catalog browser
├── astro.config.mjs
└── package.json

nightwatch-astro/scoop-bucket/                # Scoop distribution
└── bucket/astro-up.json
```

**Monorepo approach:** Cargo workspace with 3 packages, each containing crates:

| Package (Cargo.toml) | Crates inside | Purpose |
|---|---|---|
| `astro-up-core` | 1 library crate (`lib.rs`) | Shared logic: types, detect, download, install, engine, catalog, providers |
| `astro-up-cli` | 1 library + 1 binary (`lib.rs` + `main.rs`) | CLI: clap + ratatui. `lib.rs` for testable logic, `main.rs` for entry point |
| `astro-up-gui` | 1 library + 1 binary (`lib.rs` + `main.rs`) | GUI: Tauri + Vue. `lib.rs` for testable commands, `main.rs` for app init |

Inside `astro-up-core`, `types/`, `detect/`, `download/`, `install/`, `engine/`, `catalog/`, `providers/` are **modules** (not separate packages). They share one `Cargo.toml` and one compilation unit. When compile times justify it, any module can be extracted into its own package — move the directory to `crates/astro-up-{name}/`, add a `Cargo.toml`, update imports. The public API doesn't change because the module boundaries are already clean.

## Rust Dependencies

Reference: [blessed.rs](https://blessed.rs/crates) as the authoritative source for vetted Rust libraries.

### Core Dependencies (astro-up-core)

| Crate | Version | Purpose | Replaces (Go) |
|-------|---------|---------|---------------|
| **anyhow** | 1 | Error handling with context (application code) | `fmt.Errorf("...: %w", err)` |
| **thiserror** | 2 | Typed error enums (library code) | `core/errors.go` sentinels |
| **serde** | 1 (derive) | Serialization for TOML, JSON, config | `encoding/json`, `go-toml` |
| **serde_json** | 1 | JSON parsing (manifests.json, API responses) | `encoding/json` |
| **toml** | 0.8 | TOML manifest parsing | `go-toml/v2` |
| **tokio** | 1 (full) | Async runtime (HTTP, WMI, concurrent downloads) | goroutines |
| **tracing** | 0.1 | Structured logging (Tauri uses tracing internally) | `slog` |
| **chrono** | 0.4 (serde) | Timestamps, cache TTL, version dates | `time` |
| **semver** | 1 (serde) | Version parsing and comparison | `go-version` |
| **regex** | 1 | Version extraction from vendor pages | `regexp` |
| **once_cell** | 1 | Lazy statics (config, public key, compiled regex) | `sync.Once` |
| **itertools** | 0.14 | Iterator extensions (chunks, sorted_by, group_by) | manual loops |
| **validator** | 0.20 (derive) | Struct field validation | `go-playground/validator` |
| **directories** | 6 | Platform-aware config/cache/data dirs | `{config_dir}` expansion |
| **derive_more** | 2 (display, from) | Derive Display, From, Into for wrapper types | manual implementations |
| **reqwest** | 0.12 (stream, json) | HTTP client for downloads and API calls | `net/http` |
| **sha2** | 0.10 | SHA256 hash verification for downloads | `crypto/sha256` |
| **minisign-verify** | 0.2 | Minisign signature verification (by minisign creator) | `go-minisign` |
| **scraper** | 0.22 | HTML parsing for vendor page scraping | `goquery` |
| **walkdir** | 0.2 | Directory traversal (backup paths, manifest scanning) | `filepath.Walk` |
| **pelite** | 0.10 | PE file version extraction (cross-platform!) | `debug/pe` + Windows API |

**Windows-only (behind `cfg(windows)` / optional features):**

| Crate | Version | Purpose |
|-------|---------|---------|
| **winreg** | 0.56 | Windows registry access for detection |
| **wmi** | 0.18 | WMI queries for driver detection |

### CLI Dependencies (astro-up-cli)

| Crate | Version | Purpose | Replaces (Go) |
|-------|---------|---------|---------------|
| **clap** | 4 (derive) | CLI argument parsing | `urfave/cli` |
| **ratatui** | 0.29 | TUI progress bars and status display | `bubbletea/v2` |
| **indicatif** | 0.17 | Simple progress bars (alternative to ratatui for basic cases) | `bubbletea` |
| **dialoguer** | 0.11 | Interactive prompts (confirm, select, input) | manual stdin |
| **console** | 0.15 | Terminal styling (colors, bold, width detection) | `lipgloss` |

### GUI Dependencies (astro-up-gui)

Tauri v2 + official plugins (see Tauri Plugins section above).

### Dev Dependencies (all crates)

| Crate | Version | Purpose |
|-------|---------|---------|
| **insta** | 1 (json, toml) | Snapshot testing — compare CLI output, JSON responses |
| **pretty_assertions** | 1 | Diff display in test failures |
| **tempfile** | 3 | Temporary directories for downloads, test fixtures |
| **tokio-test** | 0.4 | Async test utilities |

### Worth Considering (add when needed)

| Crate | Version | Use case | When to add |
|-------|---------|----------|-------------|
| **rayon** | 1 | Parallel manifest checking (CPU-bound regex) | Only if sequential checking is too slow. Tokio handles I/O concurrency. |
| **proptest** | 1 | Property-based testing (fuzz version parsing, manifest deser) | When version parsing edge cases become a problem |
| **derive_builder** | 0.20 | Builder pattern for complex option structs | If `InstallOptions`, `CheckverConfig` constructors get unwieldy. Struct literals may be enough. |
| **fakeit** | 1 | Fake data generation for test manifests | When writing large-scale integration tests |

## Spec Breakdown

### Phase 0: Foundation

#### Spec 001 — Repository Scaffolding

**Scope:** Create the GitHub repo, Cargo workspace, Tauri project, Vue frontend, CI, branch protection, and initial configuration.

**Deliverables:**
- GitHub repo `nightwatch-astro/astro-up` with Apache-2.0 license
- Cargo workspace with three crates (astro-up-core, astro-up-cli, astro-up-gui)
- Tauri v2 project in astro-up-gui with `tauri.conf.json`
- Vue 3 + PrimeVue + VueQuery frontend scaffolding with Vite
- GitHub Actions CI: `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check`, Vue lint/test
- Branch protection on `main`: require CI + PR review
- Speckit initialized (`.specify/`)
- CLAUDE.md with project conventions
- Copy `research/debate-architecture-rewrite-rust-tauri.md` and this migration plan to `research/`
- Dependabot / renovate for dependency updates
- release-please or cargo-release setup
- `.gitignore` for Rust + Node + Tauri artifacts

---

### Phase 1: Core Domain (astro-up-core)

#### Spec 002 — Core Domain Types

**Scope:** Define the shared types, traits, and error types that all other crates depend on. This is the `astro-up-core` crate's foundation.

**Key types to define:**
- `Software` — ID, slug, name, category, OS, description, homepage, requires, optional_addons
- `Category` — 10-category enum (capture, guiding, platesolving, equipment, focusing, planetarium, viewers, prerequisites, usb, driver)
- `DetectionConfig` — method (registry, pefile, wmi, driver_store, ascom_profile, file_exists), keys, fallback chain
- `InstallConfig` — method (exe, msi, innosetup, zip, zipwrap), quiet_args, interactive_args, install_dir, post_install
- `RemoteConfig` / `CheckverConfig` — provider, owner, repo, checkver URL + regex/jsonpath pattern, autoupdate URL template
- `DownloadConfig` — resolver steps (template, redirect, scrape)
- `BackupConfig` — config_paths to preserve
- `VersioningConfig` — side-by-side, major version pattern, overrides
- `UpdatePolicy` — default + per-package overrides (minor, major, manual, none)
- Error types: `ErrNotInstalled`, `ErrChecksumMismatch`, `ErrProviderUnavailable`, `ErrManifestInvalid`, `ErrInstallerFailed` (with exit code), `ErrElevationRequired`, `ErrRebootRequired`, `ErrInstallerTimeout`, `ErrInstallerBusy`
- Traits: `Detector`, `Installer`, `Provider`, `Downloader`

**Reference:** Current Go types in `internal/core/` (software.go, config.go, interfaces.go, errors.go)

#### Spec 003 — Configuration System

**Scope:** Layered configuration loading: defaults → TOML file → environment variables.

**Key decisions:**
- Use `figment` crate (by Rocket author) or `config` crate for layered config
- TOML format matching current Go koanf tags
- `ASTROUP_*` env var prefix with nested mapping (e.g., `ASTROUP_GITHUB_TOKEN` → `github.token`)
- Validation via `validator` crate
- Config path resolution: `{config_dir}/astro-up/config.toml`
- Platform-aware path expansion (`{program_dir}`, `{config_dir}`, `{cache_dir}`)

**Reference:** Current Go config in `internal/config/config.go`

#### Spec 004 — Manifest Parsing and Catalog

**Scope:** Parse TOML manifests, compile to catalog, load from signed JSON.

**Key decisions:**
- Adopt `[checkver]` section (renamed from `[remote]`) with Scoop-style `$version` variables
- Default installer switches per type (InnoSetup: `/VERYSILENT /NORESTART /SUPPRESSMSGBOXES` unless overridden)
- Three-tier cache: TTL memory → disk → ETag network
- Minisign signature verification using `minisign-verify` crate
- SHA256 hashes in version entries
- `meta.json` lightweight change detection (100 bytes, checked before full download)
- Catalog operations: resolve by ID, resolve by slug, fuzzy search, OS filter, category filter

**Reference:** Current Go catalog in `internal/catalog/catalog.go`, `internal/config/manifest.go`

---

### Phase 2: Detection

#### Spec 005 — Windows Registry and PE Detection

**Scope:** Detect installed software via Windows registry (uninstall keys, ASCOM Profile) and PE file version info.

**Key decisions:**
- `winreg` crate for registry access
- `pelite` crate for PE version extraction (cross-platform — works in Linux CI!)
- Detection methods: `registry` (HKLM/HKCU uninstall keys + DisplayVersion), `pefile` (VS_FIXEDFILEINFO), `configfile` (regex on config files), `file_exists`
- Fallback chain: primary → fallback detection config
- ASCOM Profile: `HKLM\SOFTWARE\ASCOM\{device_type}\{driver_id}`
- Version parsing via `semver` crate

**Reference:** Current Go detect in `internal/detect/` (registry_windows.go, pefile_windows.go, composite.go)

#### Spec 006 — WMI Driver Detection

**Scope:** Detect installed drivers via WMI queries (`Win32_PnPSignedDriver`).

**Key decisions:**
- `wmi` crate with typed serde deserialization
- Query `Win32_PnPSignedDriver` by provider name, device class, INF name
- Detection config: `method = "driver_store"`, `driver_provider`, `driver_class`
- Async support via `wmi` crate's async API
- Map USB VID:PID to known astrophotography devices

**Reference:** Deferred issue #349, research in debate document

---

### Phase 3: Remote Providers and Version Checking

#### Spec 007 — Remote Version Providers

**Scope:** Check latest versions from GitHub, GitLab, and vendor websites.

**Key decisions:**
- `octocrab` crate for GitHub API (LatestRelease, ListReleases)
- `reqwest` for GitLab API and direct URL providers
- `scraper` crate for HTML scraping (replaces goquery)
- Declarative `[checkver]` patterns: `github` shorthand, URL + regex, URL + jsonpath + regex
- Rod/headless browser: defer to manifest repo CI (not in client)
- Rate limiting: respect GitHub API limits, token-authenticated requests

**Reference:** Current Go providers in `internal/provider/` (github.go, gitlab.go), `internal/scrape/`

#### Spec 008 — ASCOM Alpaca Client

**Scope:** Query ASCOM devices and drivers via the Alpaca HTTP/JSON API.

**Key decisions:**
- Use `ascom-alpaca-rs` crate (or `ascom-alpaca-core` for types)
- Alpaca discovery: mDNS or known port (11111)
- Query connected devices, driver versions, device capabilities
- No COM interop — ASCOM Platform bridges COM↔Alpaca transparently
- Fallback: if Alpaca unavailable, detect ASCOM Profile via registry (Spec 005)

**Reference:** `ascom-alpaca-rs` crate (23 stars), ASCOM Alpaca specification

---

### Phase 4: Download and Install

#### Spec 009 — Download Manager

**Scope:** HTTP downloads with hash verification, progress reporting, and resume support.

**Key decisions:**
- `reqwest` with streaming response for progress callbacks
- SHA256 verification via `sha2` crate: compare against hash from versions.json
- Download to temp file, verify, move to destination
- ETag/Last-Modified for conditional requests
- Resume via Range headers
- Progress reporting via callback/channel (feeds Tauri events or CLI progress bar)

**Reference:** Current Go download in `internal/download/manager.go`

#### Spec 010 — Installer Execution

**Scope:** Execute silent installers (EXE/NSIS, MSI, InnoSetup, ZIP) with exit code interpretation.

**Key decisions:**
- Use `tauri-plugin-shell` (GUI) or `std::process::Command` (CLI) for execution
- Per-type default switches: InnoSetup `/VERYSILENT /NORESTART /SUPPRESSMSGBOXES`, MSI `/qn /norestart`
- Pass `InstallDir` to installers: NSIS `/D=`, InnoSetup `/DIR=`, MSI `INSTALLDIR=`
- Exit code interpretation per installer type (NSIS 0/1/2, InnoSetup 0-8, MSI 0/1602/1603/1618/3010, Windows 5/740/1223)
- Map to structured error types: `ErrInstallerCancelled`, `ErrElevationRequired`, `ErrRebootRequired`, `ErrInstallerBusy`, `ErrInstallerTimeout`
- Per-installer timeout (configurable, default 10 min)
- Admin elevation: proactive check, reactive retry on 740, user-controlled flag
- ZIP extraction via `zip` crate with zip-slip guard
- UniGetUI patterns: operation queue, auto-retry with elevation, capabilities struct

**Reference:** Current Go install in `internal/install/`, deferred issue #348, exit code research

#### Spec 011 — Install Orchestration Engine

**Scope:** Coordinate check → download → backup → install cycle with dependency resolution.

**Key decisions:**
- Dependency graph: prerequisites before dependents (e.g., ASCOM before drivers)
- Update policy enforcement: minor-only blocks major upgrades unless `--allow-major`
- Dry-run mode: report what would happen without executing
- Event system: emit events for check_started/progress/complete, download_started/progress/complete, install_started/complete
- Version cache: skip re-testing if same version already verified
- Quiet vs interactive mode: `Quiet *bool` (nil = default behavior)

**Reference:** Current Go engine in `internal/engine/` (engine.go, check.go, update.go)

---

### Phase 5: Backup and Custom Tools

#### Spec 012 — Backup and Restore

**Scope:** Backup and restore application configuration files.

**Key decisions:**
- Backup manager: archive manifest-defined config paths per software
- Zip-based backup format with timestamps
- List, restore, prune operations
- Skip missing paths with warning

**Reference:** Current Go backup in `internal/backup/`

#### Spec 013 — Custom Tools

**Scope:** Add/remove user-defined tools from GitHub URLs.

**Key decisions:**
- Parse GitHub repo URL → fetch latest release → filter Windows assets
- Auto-detect install method from asset filename (.msi, .exe, .zip)
- Generate manifest TOML, save to custom tools directory
- Confirm flow: show candidates, user picks asset, save manifest

**Reference:** Current Go custom in `internal/custom/`

---

### Phase 6: CLI

#### Spec 014 — CLI Interface

**Scope:** `astro-up-cli` binary using `clap` for argument parsing and `ratatui` for TUI progress.

**Key decisions:**
- Subcommands: `list [--json]`, `check [--json]`, `update [<id>|--all] [--allow-major] [--dry-run] [--json]`, `scan [--json] [--category]`, `add <url>`, `remove <id>`, `self-update [--dry-run]`
- Global flags: `--verbose`, `--quiet`, `--config <path>`
- Output: styled text (lipgloss-like via `colored`/`owo-colors`) or JSON
- Progress: `ratatui` for download/install progress bars
- Exit codes: 0 success, 1 error, 2 user cancelled
- Standalone binary — no Tauri/WebView dependency

**Reference:** Current Go CLI in `cmd/astro-up/cli.go`, urfave/cli + Bubble Tea

---

### Phase 7: GUI

#### Spec 015 — Tauri App Shell

**Scope:** Tauri v2 application setup, plugin configuration, window management, system tray.

**Key decisions:**
- Tauri plugins: shell, updater, notification, store, fs, http, global-shortcut, single-instance, autostart, window-state, dialog, log, os, process, opener, cli
- Tauri commands: list_software, check_updates, update_software, get_config, save_config, scan_installed, add_custom_tool, remove_custom_tool, check_self_update
- Event bridge: Rust → frontend events for progress, status changes
- System tray: update count badge, quick actions
- Window state persistence via tauri-plugin-window-state
- Self-update via tauri-plugin-updater with Ed25519 signatures

**Reference:** Tauri v2 plugin documentation, current Wails setup

#### Spec 016 — Vue Frontend

**Scope:** Vue 3 + PrimeVue + VueQuery frontend application.

**Key decisions:**
- PrimeVue DataTable for software list (sort, filter, group by category)
- PrimeVue Badge for status (up_to_date, update_available, major_upgrade, not_installed, manual_download, error)
- PrimeVue ProgressBar for download/install progress
- PrimeVue Toast for notifications
- PrimeVue form components for Settings page
- VueQuery composables wrapping Tauri `invoke()` calls
- Dark mode via PrimeVue theme (Aura Dark)
- Views: Dashboard (DataTable), Settings (form), Custom Tools
- No router needed — `ref('dashboard')` for 3-4 views
- Keyboard shortcuts via tauri-plugin-global-shortcut

**Reference:** Current Svelte frontend, PrimeVue documentation

---

### Phase 8: Distribution and Release

#### Spec 017 — CI Pipeline

**Scope:** GitHub Actions CI for Rust + Vue testing and Windows integration tests.

**Key decisions:**
- Rust: `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check`
- Vue: `pnpm lint`, `pnpm test` (Vitest), `pnpm build`
- Tauri: `cargo tauri build` on Windows runner
- Integration tests: `cargo test --features integration` on Windows
- Pre-release verification workflow (workflow_dispatch, per-package filter)
- Conventional commits (cocogitto)
- Reuse `nightwatch-astro/.github` org-level workflows where applicable

#### Spec 018 — Release Pipeline

**Scope:** Automated releases via Tauri bundler.

**Key decisions:**
- Tauri NSIS bundler for Windows installer (replaces goreleaser + Inno Setup)
- Drop portable build — Tauri installer is the distribution method
- Ed25519 signing for auto-updater
- SignPath.io for Windows Authenticode code signing (deferred)
- Release-please for version management and changelogs
- Scoop bucket update on release
- Update endpoint JSON for tauri-plugin-updater

---

### Phase 9: Migration Completion

#### Spec 019 — Manifest Pipeline Simplification

**Scope:** Update the shared manifest repository to support the new `[checkver]` format and simplified CI.

**Key decisions:**
- Rename `[remote]` → `[checkver]` in TOML manifests
- Adopt Scoop `$version`, `$majorVersion`, `$cleanVersion` template variables
- Add `sha256` field to version entries in versions.json
- Add `meta.json` for lightweight change detection
- Collapse CI matrix to single `CheckAll()` job
- Default installer switches per type (reduce manifest verbosity)
- Backward compatible: old Go client can still read manifests.json

**Note:** This spec applies to `astro-up/astro-up-manifests`, not the new Rust repo. Both old and new clients consume the same output.

#### Spec 020 — Feature Parity Verification

**Scope:** Verify the Rust implementation matches Go feature parity before switching users.

**Key decisions:**
- Comparison matrix: every CLI command, every GUI feature, every detection method
- Integration test parity: port key Go integration tests to Rust
- Manifest compatibility: both clients produce identical results for the same manifests
- Performance comparison: startup time, scan time, download speed
- User acceptance testing with real astrophotography setup

---

## Dependency Graph

```
001 (scaffold)
 ├── 002 (core types) ──┐
 ├── 003 (config)       ├── 004 (catalog) ──┐
 │                      │                    ├── 007 (providers) ──┐
 ├── 005 (registry/PE) ─┤                    │                     ├── 011 (engine) ──┐
 ├── 006 (WMI drivers) ─┘                    ├── 009 (download) ───┤                  │
 │                                           │                     ├── 010 (install) ─┤
 │                                           ├── 008 (Alpaca) ─────┘                  │
 │                                           │                                        │
 │                                           ├── 012 (backup) ────────────────────────┤
 │                                           ├── 013 (custom tools) ──────────────────┤
 │                                           │                                        │
 │                                           ├── 014 (CLI) ───────────────────────────┤
 │                                           │                                        │
 │                                           ├── 015 (Tauri shell) ───────────────────┤
 │                                           ├── 016 (Vue frontend) ──────────────────┤
 │                                           │                                        │
 │                                           ├── 017 (CI) ────────────────────────────┤
 │                                           └── 018 (release) ───────────────────────┘
 │
 └── 019 (manifest pipeline) ── independent, applies to manifest repo
     020 (parity verification) ── after all other specs
```

## Agent Task Instructions

Each spec should be written by a speckit agent using `/speckit.specify`. The agent receives:

1. This migration plan document
2. The architecture decision document (`research/debate-architecture-rewrite-rust-tauri.md`)
3. The specific spec section from this plan
4. Access to the old Go codebase for reference (`astro-up/astro-up`)

The agent should:
- Read the relevant Go source files for the feature being specified
- Translate Go types/interfaces to Rust equivalents
- Reference specific Rust crates with version numbers
- Define acceptance criteria based on the Go implementation's behavior
- Include integration test requirements
- Note any deviations from the Go implementation (and why)

## Execution Order

1. **Spec 001** (scaffold) — must be first, creates the repo and workspace
2. **Specs 002-003** (core types, config) — foundation, can be parallel
3. **Spec 004** (catalog) — depends on 002+003
4. **Specs 005-008** (detection, providers) — can be parallel after 002
5. **Specs 009-011** (download, install, engine) — after providers
6. **Specs 012-013** (backup, custom) — after engine
7. **Spec 014** (CLI) — after engine
8. **Specs 015-016** (GUI) — after engine
9. **Specs 017-018** (CI, release) — after GUI
10. **Spec 019** (manifest pipeline) — independent, anytime
11. **Spec 020** (parity) — last
