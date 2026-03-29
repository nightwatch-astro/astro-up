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
| **tracing** | 0.1 | Structured logging + spans (Tauri uses tracing internally) | `slog` |
| **metrics** | 0.24 | Metrics facade (counters, gauges, histograms) for UI display + future telemetry | — |
| **metrics-util** | 0.18 | In-memory recorder for exposing metrics to Tauri UI | — |
| **chrono** | 0.4 (serde) | Timestamps, cache TTL, version dates | `time` |
| **semver** | 1 (serde) | Version parsing and comparison | `go-version` |
| **regex** | 1 | Version extraction from vendor pages | `regexp` |
| ~~once_cell~~ | — | Replaced by `std::sync::LazyLock` (stable since Rust 1.80) | `sync.Once` |
| **itertools** | 0.14 | Iterator extensions (chunks, sorted_by, group_by) | manual loops |
| **figment** | 0.10 (toml, env) | Layered config: defaults → TOML → env vars | `koanf` |
| **garde** | 0.22 (derive) | Struct field validation — cleaner derive, automatic nested validation, async support | `go-playground/validator` |
| **flume** | 0.11 | Sync+async channels for engine → UI event bridge (CLI sync recv, GUI async recv) | `std::sync::mpsc` + goroutine channels |
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
| **anyhow** | 1 | Application error handling (CLI uses anyhow + console for colored output) | `fmt.Errorf` |
| **ratatui** | 0.29 | TUI progress bars and status display | `bubbletea/v2` |
| **indicatif** | 0.17 | Simple progress bars for downloads | `bubbletea` |
| **inquire** | 0.7 | Interactive prompts (fuzzy select, validation) | manual stdin |
| **console** | 0.15 | Terminal styling (colors, bold, width detection) | `lipgloss` |
| **tabled** | 0.15 | Formatted table output for `list`, `check`, `scan` | `lipgloss` tables |
| **tracing-subscriber** | 0.3 (env-filter) | Log output formatting and filtering | `slog` handlers |

### GUI Dependencies (astro-up-gui)

Tauri v2 + official plugins (see Tauri Plugins section above).

### Dev Dependencies (all crates)

| Crate | Version | Purpose |
|-------|---------|---------|
| **insta** | 1 (json, toml) | Snapshot testing — compare CLI output, JSON responses |
| **pretty_assertions** | 1 | Diff display in test failures |
| **tempfile** | 3 | Temporary directories for downloads, test fixtures |
| **tokio-test** | 0.4 | Async test utilities |
| **rstest** | 0.23 | Parameterized tests for version parsing, manifest deserialization |
| **wiremock** | 0.6 | HTTP server mocking for provider integration tests |

### Changes from Initial List

| Crate | Change | Reason |
|-------|--------|--------|
| ~~color-eyre~~ | Removed — using anyhow + console for colored error output instead | eyre ecosystem fragmentation risk; anyhow is the gold standard |
| **garde** | Replaces validator | Cleaner derive, automatic nested validation, async support |
| **flume** | Replaces tokio::sync::mpsc for event channels | Dual sync/async API — CLI can recv() without tokio runtime |
| **inquire** | Replaces dialoguer | Better UX: fuzzy select, custom validation, polished prompts |
| **strum** | Added | Enum derive macros (Display, EnumString, EnumIter) for Category, Status enums |
| **futures** | Added | Stream combinators for concurrent downloads |
| **globset** | Added | File glob matching for asset patterns (more natural than regex for `*win64.exe`) |
| **serde_with** | Added | Custom serde helpers (Duration as string, skip None) |
| **cargo-nextest** | Added dev tool | Faster test runner, blessed.rs recommended |
| **install-action** | Added CI tool | GitHub Action for installing Rust tools |

### Worth Considering (add when needed)

| Crate | Version | Use case | When to add |
|-------|---------|----------|-------------|
| **rayon** | 1 | Parallel manifest checking (CPU-bound regex) | Only if sequential checking is too slow. Tokio handles I/O concurrency. |
| **proptest** | 1 | Property-based testing (fuzz version parsing, manifest deser) | When version parsing edge cases become a problem |
| **derive_builder** | 0.20 | Builder pattern for complex option structs | If `InstallOptions`, `CheckverConfig` constructors get unwieldy. Struct literals may be enough. |
| **fakeit** | 1 | Fake data generation for test manifests | When writing large-scale integration tests |
| **notify** | 7 | File system watcher for live config reload, custom tool directory changes | When implementing live config reload |
| **parking_lot** | 0.12 | Faster Mutex/RwLock (no poisoning) | Only if std::sync::Mutex contention becomes measurable |
| **globset** | 0.4 | Glob matching for asset patterns | If regex feels unnatural for file pattern matching in manifests |
| **sentry** | 0.34 (tracing feature) | Error reporting + performance traces | When telemetry feature is implemented. Opt-in, user consent required. `sentry-tracing` integrates with existing tracing spans. |
| **posthog** | — (HTTP API via reqwest) | Product analytics (feature usage, update patterns) | When telemetry feature is implemented. Opt-in, user consent required. No official Rust SDK — use `reqwest` POST to `/capture`. |
| **flume** | 0.11 | Faster async+sync channels | If `tokio::sync::mpsc` performance is insufficient for event streaming |

### Added Since Initial Plan

| Crate | Version | Purpose |
|-------|---------|---------|
| **rusqlite** | 0.32 (bundled) | Embedded SQLite for catalog, version history, ledger, config, ignored updates. Replaces three-tier cache, fuzzy search crate, JSON parsing, tauri-plugin-store for config, and deferred ledger feature (#347) |
| **zip** | 2 | ZIP extraction for wrapped installers (includes flate2 internally) |

### Explicitly Skipped

| Crate | Reason |
|-------|--------|
| postcard | Serde format for embedded/no_std wire size — we read TOML/JSON, not binary |
| rocksdb | SQLite covers all our needs with less complexity |
| axum | Web framework — we're not serving HTTP, Tauri invoke() handles IPC |
| ureq | Blocking HTTP — need async streaming with progress for downloads |
| argon2/bcrypt | Password hashing — no passwords in this app |
| blake3 | We verify vendor SHA256 hashes, not our own hashes |
| crossbeam-channel/flume/postage | tokio::sync::mpsc covers async channels |
| async-trait | Native async traits since Rust 1.75 |
| cargo-zigbuild | No cross-compilation needed (Windows → Windows) |
| signal-hook | Windows doesn't use Unix signals, Tauri handles lifecycle |
| tap/joinery/conv/educe/variantly/soa_derive | Niche — superseded by itertools, derive_more, strum, or stdlib |

## Revised Data Model Decisions (post-debate)

**SQLite replaces JSON caching.** Client uses `rusqlite` (bundled) for: catalog, version history, installed state, config, ignored updates, and the fallback ledger. Replaces three-tier cache, fuzzy search crate, JSON parsing, and the deferred ledger (#347).

**Version history via per-version files.** The manifest repo stores discovered versions as individual files per product:
```
versions/nina-app/3.0.0.json    # { url, sha256, discovered_at, release_notes_url }
versions/nina-app/3.1.0.json
versions/phd2/2.6.14.json
```
The compiler produces a single `versions.json` for client download. Client imports into SQLite. Git history provides full audit trail. Enables rollback to any previously-discovered version.

**Pin URL + SHA256 for all versions at discovery time.** CI checker computes SHA256 when discovering a new version. Sources: GitHub/GitLab API checksums, vendor checksum files (`hash_url` + `hash_regex`, Scoop pattern), or full download + compute for new versions only. Dynamic URLs resolved and pinned at discovery time.

**Hybrid URL model.** ~64% predictable (pinned statically), ~9% dynamic (pinned at discovery time), ~27% download-only (pinned where possible).

**Download-only install method.** `install.method = "download_only"` for firmware. Downloads to configured directory, notifies user. No ledger for download state — uninstalled downloads still show as outdated.

**Driver sub-categories:**

| Type | Install | Detection | Notes |
|------|---------|-----------|-------|
| Driver packs (ZWO, QHY) | Full install | Registry + WMI | Works without hardware |
| ASCOM drivers | Full install | ASCOM Profile | Works without hardware |
| USB serial (FTDI, CP210x) | Full install | WMI driver store | Works without hardware |
| Firmware files | Download only | Never (device-side) | User acknowledges in ledger |
| Firmware updater tools | Full install, `upgrade_behavior = "deny"` | Registry | Prevent auto-upgrade (bricking risk) |

**Fallback ledger.** SQLite table for undetectable packages and manual tracking:

```sql
CREATE TABLE ledger (
    package_id TEXT NOT NULL PRIMARY KEY,
    version TEXT NOT NULL,
    source TEXT NOT NULL,        -- 'astro-up' | 'manual' | 'acknowledged'
    recorded_at TEXT NOT NULL,
    notes TEXT
);
```

Priority: auto-detection > ledger > unknown. If they disagree, warn the user. Supports firmware acknowledgment, manual version entry, and future compatibility for any untrackable software.

**Device connection notifications (Phase 3).** WMI event subscription for USB connect. Match VID:PID against known devices. Notify if update available. Debounced, user-configurable.

**License awareness.** `license` + `license_url` fields. First-install GUI prompt. `--accept-agreements` CLI flag. Tracked per-package in SQLite.

**Portable installer type.** `install.method = "portable"` for standalone executables. Extract to directory, optionally add to PATH.

**Scope model.** `scope = "machine" | "user" | "either"`. Prompted in CLI (or via `--scope` flag) and GUI dialog when `either`. Setting for default scope preference. Error if flag contradicts manifest requirement.

**Setup wizard (future spec, Phase 3).** Equipment-based guided install: "What mount? What camera?" → recommended package set → one-click install with dependency ordering.

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

**Scope:** Define the shared types, traits, and error types that all other crates depend on. This is the `astro-up-core` crate's foundation. Data model informed by winget's schema (v1.9.0) — adopt fields that improve UX and correctness, skip fields that only serve winget's scale or Windows Store.

**Software metadata (winget-inspired enrichments):**
- `Software` — ID, slug, name, category, OS, description, homepage, publisher, icon, license, aliases, tags, notes, docs_url, channel, min_os_version, requires (with min_version), optional_addons
- `Category` — 10-category enum (capture, guiding, platesolving, equipment, focusing, planetarium, viewers, prerequisites, usb, driver)

**Detection:**
- `DetectionConfig` — method (registry, pefile, wmi, driver_store, ascom_profile, file_exists), keys, fallback chain, product_code, upgrade_code

**Install (winget switch model):**
- `InstallConfig` — method (exe, msi, innosetup, nullsoft, wix, burn, zip, zipwrap, portable), scope (machine/user/either), elevation (required/prohibited/self), upgrade_behavior (install/uninstall_previous/deny), install_modes (interactive/silent), success_codes, pre_install, post_install
- `InstallerSwitches` — silent, interactive, upgrade, install_location (with `<INSTALLPATH>` token), log (with `<LOGPATH>` token), custom
- `KnownExitCodes` — map of exit code → semantic meaning (packageInUse, rebootRequired, cancelledByUser, alreadyInstalled, missingDependency, etc.)

**Remote / Checkver:**
- `CheckverConfig` — provider, owner, repo, checkver URL + regex/jsonpath pattern, autoupdate URL template
- `DownloadConfig` — resolver steps (template, redirect, scrape)

**Config / Policy:**
- `BackupConfig` — config_paths to preserve (no persist/symlinks — avoids lock-in and brownfield issues)
- `VersioningConfig` — side-by-side, major version pattern, overrides
- `UpdatePolicy` — default + per-package overrides (minor, major, manual, none)

**Error types:**
- `NotInstalled`, `ChecksumMismatch`, `ProviderUnavailable`, `ManifestInvalid`
- `InstallerFailed { exit_code, response }` — wraps `KnownExitCode` semantic
- `ElevationRequired`, `RebootRequired`, `InstallerTimeout`, `InstallerBusy`
- `PackageInUse { process_name }`, `AlreadyInstalled`, `MissingDependency { dep_id }`

**Traits:** `Detector`, `Installer`, `Provider`, `Downloader`

**Example manifest (new format):**

```toml
id = "nina-app"
name = "N.I.N.A."
type = "application"
category = "capture"
publisher = "Stefan Berg (isbeorn)"
description = "Nighttime Imaging 'N' Astronomy — advanced capture sequencer"
homepage = "https://nighttime-imaging.eu"
icon = "https://nighttime-imaging.eu/icon.png"
license = "MPL-2.0"
aliases = ["nina"]
tags = ["sequencer", "dso", "ascom-compatible", "plate-solving", "autofocus"]
notes = "Requires .NET Desktop Runtime 8 and ASCOM Platform"
channel = "stable"
min_os_version = "10.0"

[dependencies]
requires = [
    { id = "ascom-platform", min_version = "6.6" },
    { id = "dotnet-desktop-8" },
]
optional = ["phd2-guider"]

[detection]
method = "registry"
registry_key = "SOFTWARE\\NINA"
registry_value = "DisplayVersion"

[detection.fallback]
method = "pe_file"
file_path = "{program_dir}/NINA/NINA.exe"

[install]
method = "innosetup"
scope = "either"
elevation = "self"
upgrade_behavior = "install"
install_modes = ["interactive", "silent"]
success_codes = [3010]
pre_install = []
post_install = []

[install.switches]
silent = ["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"]
interactive = ["/NORESTART"]
upgrade = ["/VERYSILENT", "/NORESTART", "/SUPPRESSMSGBOXES"]
install_location = "/DIR=<INSTALLPATH>"
log = "/LOG=<LOGPATH>"

[install.known_exit_codes]
1 = "package_in_use"
6 = "package_in_use_by_application"
3010 = "reboot_required"

[checkver]
github = "isbeorn/nina"
asset_pattern = "NINASetupBundle_*.zip"
tag_prefix = "Version-"
changelog_url = "https://github.com/isbeorn/nina/releases"

[backup]
config_paths = [
    "{config_dir}/NINA/Profiles",
    "{config_dir}/NINA/Settings",
]

# For driver packages only:
[hardware]
vid_pid = ["03C3:*"]             # ZWO — match all products under vendor ID
device_class = "Camera"          # Windows device class for WMI matching
inf_provider = "ZWO"             # Match Win32_PnPSignedDriver.DriverProviderName
```

**Hardware section:** Only for driver manifests. Used for:
1. Device connection notifications (match USB VID:PID on connect event)
2. WMI driver detection (filter `Win32_PnPSignedDriver` by provider/class)
3. Driver verification when hardware is connected

**VID:PID discovery automation:** A `just` task downloads driver packages, extracts INF files, and parses VID:PID declarations:

```just
# Justfile (manifest repo)
discover-vid-pid package_id:
    #!/usr/bin/env bash
    set -euo pipefail
    # Download latest driver package
    url=$(jq -r '.["{{package_id}}"].url' versions.json)
    tmpdir=$(mktemp -d)
    curl -L -o "$tmpdir/driver.exe" "$url"
    # Extract (handles ZIP, NSIS self-extracting, etc.)
    7z x "$tmpdir/driver.exe" -o"$tmpdir/extracted" -y 2>/dev/null || \
        unzip "$tmpdir/driver.exe" -d "$tmpdir/extracted" 2>/dev/null || true
    # Find INF files and extract VID:PID
    grep -rhiE 'USB\\VID_[0-9A-Fa-f]{4}&PID_[0-9A-Fa-f]{4}' \
        "$tmpdir/extracted/" --include='*.inf' | \
        grep -oiE 'VID_[0-9A-Fa-f]{4}&PID_[0-9A-Fa-f]{4}' | \
        sort -u | \
        sed 's/VID_//;s/&PID_/:/' | tr '[:upper:]' '[:lower:]'
    rm -rf "$tmpdir"

discover-all-vid-pids:
    @for manifest in manifests/driver/*.toml manifests/usb/*.toml; do \
        id=$(basename "$manifest" .toml); \
        echo "=== $id ==="; \
        just discover-vid-pid "$id" 2>/dev/null || echo "  (no VID:PID found)"; \
    done
```

**CI validation step:** Compare manifest `[hardware].vid_pid` against INF-extracted values. Warn on mismatch (new product IDs added by vendor, or manifest missing entries).

**Reference:** Current Go types in `internal/core/` (software.go, config.go, interfaces.go, errors.go). winget installer schema v1.9.0.

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

#### Spec 019 — Manifest Pipeline Modernization

**Scope:** Modernize the manifest repository with Scoop/winget-inspired patterns: self-describing manifests, SQLite compilation, per-version files, simplified CI, and GitHub Releases distribution.

**Key decisions:**
- Rename `[remote]` → `[checkver]` in TOML manifests (self-describing, not stripped by compiler)
- Adopt Scoop `$version`, `$majorVersion`, `$cleanVersion`, `$minorVersion`, `$patchVersion`, `$underscoreVersion`, `$dashVersion` template variables for URL construction
- Tiered hash discovery (Scoop pattern): URL+regex > JSON endpoint > download+compute
- TOML → SQLite compilation (winget pattern): replaces `manifests.json` + `versions.json` with single `catalog.db`
- Per-version files: `versions/{id}/{semver}.json` with `{ url, sha256, discovered_at, release_notes_url }` — git history is the audit trail
- Collapse CI to single job: iterate manifests → checkver → update version files → compile SQLite → publish
- Distribution via GitHub Releases: rolling `catalog/latest` tag with `catalog.db` + `catalog.db.minisig` assets. Upgrade path to Cloudflare R2 if custom cache headers needed later.
- Client fetches SQLite via HTTP GET with ETag for conditional requests
- Default installer switches per type (reduce manifest verbosity)
- Schema versioning: `manifest_version` field in every TOML file
- Keep separate repo (6h cron commits + Chromium deps stay out of main repo)
- Backward compatible transition: old Go client reads `manifests.json` (generated alongside SQLite during transition)

**Research basis:** Scoop (self-describing manifests, variable substitution, tiered hash discovery), winget (YAML→SQLite compilation, CDN distribution, schema versioning).

**Note:** This spec applies to `nightwatch-astro/astro-up-manifests`, not the main Rust repo. The main app's Spec 004 (Catalog) consumes the SQLite artifact.

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
