# Implementation Plan: Portable Apps Folder

**Branch**: `031-portable-apps-folder` | **Date**: 2026-04-12 | **Spec**: [spec.md](spec.md)

## Summary

Add a configurable portable apps directory where `download_only` and `portable` packages are placed. Currently, `download_only` (17 packages) opens Explorer on the download folder. After this change, files are copied/extracted to `{portable_apps_dir}/{package-id}/` with the path recorded in the ledger.

## Technical Context

**Language/Version**: Rust 2024 edition + Vue 3 / TypeScript 5
**Primary Dependencies**: tokio (async fs), zip 2 (extraction), directories 6.0 (existing — provides data_dir)
**Storage**: SQLite (config_settings, ledger tables — existing)
**Testing**: cargo test + vitest
**Target Platform**: Windows (primary), macOS/Linux (CI compilation)
**Project Type**: desktop-app (Tauri v2)
**Performance Goals**: Copy/extract completes within 5 seconds of download finishing
**Constraints**: No elevation for portable dir (user-owned path)
**Scale/Scope**: 17 `download_only` manifests, 0 `portable` manifests

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First | PASS | Changes in existing `install/`, `config/` modules — no new crate |
| II. Platform Awareness | PASS | Default path uses `directories` crate for platform-appropriate location; `#[cfg(windows)]` for Explorer open |
| III. Test-First | PASS | Integration tests for copy/extract, snapshot tests for config |
| IV. Thin Tauri Boundary | PASS | All logic in `astro-up-core`; GUI command passes config path through |
| V. Spec-Driven | PASS | This spec |
| VI. Simplicity | PASS | Reuses existing `handle_portable_install` pattern; no new abstractions |
| VII. Observability | PASS | `#[tracing::instrument]` on modified handlers |

## Project Structure

### Documentation

```text
specs/031-portable-apps-folder/
├── spec.md
├── plan.md              # This file
├── research.md
├── data-model.md
├── quickstart.md
└── checklists/
    └── requirements.md
```

### Source Code (files to modify)

```text
crates/astro-up-core/src/
├── config/
│   ├── model.rs          # Add portable_apps_dir to PathsConfig
│   ├── defaults.rs       # Default path
│   └── mod.rs            # config_set/config_get for new field
├── install/
│   └── mod.rs            # Modify handle_download_only, handle_portable_install
└── types/
    └── install.rs        # Add portable_apps_dir to InstallRequest (if needed)

crates/astro-up-gui/src/
└── commands.rs           # Pass portable_apps_dir to InstallerService

frontend/src/
├── types/config.ts       # Add portable_apps_dir type
├── validation/config.ts  # Add validation
├── views/SettingsView.vue # Add default config value
└── components/settings/
    └── PathsSection.vue  # Add input field
```

## Design

### Config Field

Add `portable_apps_dir: PathBuf` to `PathsConfig`. Default: `{data_dir}/../apps/` which resolves to `{AppData}/nightwatch/astro-up/apps/` on Windows (sibling to the `data/` directory).

### Install Handler Changes

**`handle_download_only`** (the big change):
1. Remove the `explorer.exe` open behavior
2. Determine if the download is a zip or single file
3. If zip: extract to `{portable_apps_dir}/{package-id}/`
4. If single file: copy to `{portable_apps_dir}/{package-id}/`
5. Return `InstallResult::Success { path: Some(dest) }`
6. This handler currently early-returns before hooks/elevation — keep the early return but add the copy logic

**`handle_portable_install`** (small change):
1. Change `resolve_install_dir` to prefer `portable_apps_dir` when the method is `Portable`
2. Or: pass `portable_apps_dir` via `InstallRequest` so the installer knows where to put it

### Passing the Config

In `commands.rs`, the `InstallerService` is created with a hardcoded temp dir. Options:
1. **Simple**: Pass `portable_apps_dir` on `InstallRequest` — the handler reads it from there
2. **Structural**: Add it to `InstallerService` constructor

Option 1 is simpler and doesn't change the service interface. The `InstallRequest` already has `install_dir: Option<PathBuf>` — for download-only/portable packages, set this to `{portable_apps_dir}/{package-id}`.

### Frontend

Add a "Portable Apps" path input to `PathsSection.vue` with a browse button (reusing the existing pattern from `download_dir`).

### Update Behavior (FR-009)

When updating a portable app that already has a ledger entry with an install path, use that same path as the target. The ledger already stores `install_path` — the orchestrator can pass it through to the `InstallRequest.install_dir`.
