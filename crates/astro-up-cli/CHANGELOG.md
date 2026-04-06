# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.0...astro-up-cli-v0.1.1) (2026-04-06)


### Features

* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* avoid catalog lock contention in global_json_flag_accepted test ([#801](https://github.com/nightwatch-astro/astro-up/issues/801)) ([21a146d](https://github.com/nightwatch-astro/astro-up/commit/21a146d4f26635341060c15b2385a6b4ffff9579))
* **cli:** rewrite self-update to use self_update crate ([#769](https://github.com/nightwatch-astro/astro-up/issues/769)) ([7250835](https://github.com/nightwatch-astro/astro-up/commit/72508357c5ab01a1289b30181f1d48978c2a9edc))
* ignore json_update_valid test (hangs on CI) ([#799](https://github.com/nightwatch-astro/astro-up/issues/799)) ([c02aa8e](https://github.com/nightwatch-astro/astro-up/commit/c02aa8e10ccdbcc8308f57ffbac17df412b3ec37))

## [Unreleased]

## [0.1.0](https://github.com/nightwatch-astro/astro-up/releases/tag/astro-up-cli-v0.1.0) - 2026-04-05

### Bug Fixes

- rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/pull/767))
- log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/pull/765))
- revert manual version bump, let release-plz manage versions
- bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish
- delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/pull/753))
- skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/pull/746))
- release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/pull/732))
- *(023)* resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/pull/729))
- *(015)* exit code 2 for cancellation, --quiet suppresses tracing, log dir path
- *(ci)* inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/pull/164))

### Documentation

- add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/pull/741))

### Features

- *(023)* complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/pull/738))
- *(024)* wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/pull/716))
- *(023)* lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/pull/727))
- docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/pull/656))
- *(015)* install, update, backup, restore, config, self-update, exit codes (T021-T035)
- *(015)* scan and search commands (T017-T020)
- *(015)* show command — catalog listing, detail, backups (T012-T016)
- *(015)* CLI scaffold — deps, modules, dispatch, logging (T001-T011)
- *(001)* T004-T009, T019 — create Rust crates (core, cli, gui)

### Miscellaneous

- release v0.1.0 ([#766](https://github.com/nightwatch-astro/astro-up/pull/766))
- reset cli version to 0.1.0
- disable crates.io publishing entirely, distribute via GitHub releases only
- release v0.2.0 ([#760](https://github.com/nightwatch-astro/astro-up/pull/760))
- release v0.1.1 ([#742](https://github.com/nightwatch-astro/astro-up/pull/742))
- release v0.1.0 ([#163](https://github.com/nightwatch-astro/astro-up/pull/163))

### Testing

- *(015)* deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/pull/462))

### Style

- *(001)* T024-T026 — fix rustfmt, verify just check passes

## [0.1.0](https://github.com/nightwatch-astro/astro-up/releases/tag/astro-up-cli-v0.1.0) - 2026-04-05

### Bug Fixes

- rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/pull/767))
- log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/pull/765))
- revert manual version bump, let release-plz manage versions
- bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish
- delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/pull/753))
- skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/pull/746))
- release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/pull/732))
- *(023)* resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/pull/729))
- *(015)* exit code 2 for cancellation, --quiet suppresses tracing, log dir path
- *(ci)* inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/pull/164))

### Documentation

- add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/pull/741))

### Features

- *(023)* complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/pull/738))
- *(024)* wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/pull/716))
- *(023)* lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/pull/727))
- docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/pull/656))
- *(015)* install, update, backup, restore, config, self-update, exit codes (T021-T035)
- *(015)* scan and search commands (T017-T020)
- *(015)* show command — catalog listing, detail, backups (T012-T016)
- *(015)* CLI scaffold — deps, modules, dispatch, logging (T001-T011)
- *(001)* T004-T009, T019 — create Rust crates (core, cli, gui)

### Miscellaneous

- reset cli version to 0.1.0
- disable crates.io publishing entirely, distribute via GitHub releases only
- release v0.2.0 ([#760](https://github.com/nightwatch-astro/astro-up/pull/760))
- release v0.1.1 ([#742](https://github.com/nightwatch-astro/astro-up/pull/742))
- release v0.1.0 ([#163](https://github.com/nightwatch-astro/astro-up/pull/163))

### Testing

- *(015)* deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/pull/462))

### Style

- *(001)* T024-T026 — fix rustfmt, verify just check passes

## [0.2.0](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.1...astro-up-cli-v0.2.0) - 2026-04-05

### Bug Fixes

- delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/pull/753))

### Features

- *(023)* complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/pull/738))

## [0.1.1](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.0...astro-up-cli-v0.1.1) - 2026-04-04

### Documentation

- add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/pull/741))

## [0.1.0](https://github.com/nightwatch-astro/astro-up/releases/tag/astro-up-cli-v0.1.0) - 2026-03-30

### Features

- *(001)* T004-T009, T019 — create Rust crates (core, cli, gui)

### Style

- *(001)* T024-T026 — fix rustfmt, verify just check passes
