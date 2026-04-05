# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/nightwatch-astro/astro-up/releases/tag/astro-up-core-v0.1.0) - 2026-04-05

### Bug Fixes

- use crate version in default user-agent fallback ([#764](https://github.com/nightwatch-astro/astro-up/pull/764))
- expose crate name constant for runtime identification
- add crate-level documentation to astro-up-core
- revert manual version bump, let release-plz manage versions
- bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish
- match process exe path filename to handle Linux comm truncation ([#762](https://github.com/nightwatch-astro/astro-up/pull/762))
- *(ci)* slim CI jobs, fix version-dependent snapshot ([#758](https://github.com/nightwatch-astro/astro-up/pull/758))
- delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/pull/753))
- update config snapshot for 0.1.1 version bump ([#745](https://github.com/nightwatch-astro/astro-up/pull/745))
- extract WMI data inside spawn_blocking for Send safety ([#748](https://github.com/nightwatch-astro/astro-up/pull/748))
- include minisign public key in core crate for publishing
- use NotInstalled variant for Ledger detection method ([#736](https://github.com/nightwatch-astro/astro-up/pull/736))
- handle Ledger variant in detection match arms ([#734](https://github.com/nightwatch-astro/astro-up/pull/734))
- release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/pull/732))
- skip packages with empty download URLs in orchestrator ([#728](https://github.com/nightwatch-astro/astro-up/pull/728))
- *(023)* resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/pull/729))
- PrimeVue deprecations + window size enforcement ([#655](https://github.com/nightwatch-astro/astro-up/pull/655))
- *(006)* PE detection resolves paths from install ledger ([#421](https://github.com/nightwatch-astro/astro-up/pull/421))
- *(013)* quality gates, retrospective, spec 003 reconciliation ([#420](https://github.com/nightwatch-astro/astro-up/pull/420))
- *(013)* quality gate findings, spec 003 reconciliation, pre-commit config ([#363](https://github.com/nightwatch-astro/astro-up/pull/363))
- resolve Windows clippy errors in WMI detection modules ([#260](https://github.com/nightwatch-astro/astro-up/pull/260))
- update error_display test after removing NotInstalled/UnsupportedPlatform ([#219](https://github.com/nightwatch-astro/astro-up/pull/219))
- *(ci)* inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/pull/164))
- *(003)* complete error Display test coverage, update spec FR-010
- *(003)* add missing BackupManager trait + enum validation tests

### Documentation

- add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/pull/741))

### Features

- *(ci)* add CLI integration tests on Windows ([#759](https://github.com/nightwatch-astro/astro-up/pull/759))
- *(023)* complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/pull/738))
- add Ledger detection method for firmware packages ([#731](https://github.com/nightwatch-astro/astro-up/pull/731))
- *(025)* file/directory browser picker for settings path fields ([#726](https://github.com/nightwatch-astro/astro-up/pull/726))
- *(024)* wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/pull/716))
- *(023)* lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/pull/727))
- *(022)* Vue 3 frontend — views, components, Tauri wiring ([#654](https://github.com/nightwatch-astro/astro-up/pull/654))
- *(012)* install orchestration engine ([#419](https://github.com/nightwatch-astro/astro-up/pull/419))
- *(011)* installer execution — silent install, exit codes, elevation, ZIP, uninstall ([#321](https://github.com/nightwatch-astro/astro-up/pull/321))
- *(013)* backup and restore — ZIP archives, selective restore, pruning ([#323](https://github.com/nightwatch-astro/astro-up/pull/323))
- *(010)* download manager — streaming, hash, resume, throttle, purge ([#259](https://github.com/nightwatch-astro/astro-up/pull/259))
- *(006)* software and driver detection system ([#214](https://github.com/nightwatch-astro/astro-up/pull/214))
- *(005)* manifest parsing and catalog ([#162](https://github.com/nightwatch-astro/astro-up/pull/162))
- *(004)* implement config system — load, API, tests (T011-T027)
- *(004)* implement config structs, defaults, and ConfigStore (T005-T010)
- *(004)* setup config module structure and dependencies (T001-T004)
- *(003)* T013-T043 — error types, traits, events, ledger, metrics
- *(003)* T001-T012 — workspace deps, enums, Version type
- *(001)* T004-T009, T019 — create Rust crates (core, cli, gui)

### Miscellaneous

- enable clippy pedantic+nursery+cargo and fix all warnings ([#771](https://github.com/nightwatch-astro/astro-up/pull/771))
- release v0.1.0 ([#768](https://github.com/nightwatch-astro/astro-up/pull/768))
- release v0.1.0 ([#766](https://github.com/nightwatch-astro/astro-up/pull/766))
- reset core version to 0.1.0
- disable crates.io publishing for core, use OIDC for CLI, remove CI on main push ([#763](https://github.com/nightwatch-astro/astro-up/pull/763))
- release v0.2.0 ([#760](https://github.com/nightwatch-astro/astro-up/pull/760))
- release v0.1.1 ([#742](https://github.com/nightwatch-astro/astro-up/pull/742))
- release v0.1.0 ([#163](https://github.com/nightwatch-astro/astro-up/pull/163))
- *(deps)* bump toml from 0.8.2 to 0.9.12+spec-1.1.0 ([#79](https://github.com/nightwatch-astro/astro-up/pull/79))
- *(deps)* bump strum from 0.26.3 to 0.28.0 ([#77](https://github.com/nightwatch-astro/astro-up/pull/77))

### Refactoring

- *(003)* remove superseded Detector trait and unused CoreError variants ([#218](https://github.com/nightwatch-astro/astro-up/pull/218))
- *(003)* align Software.id with PackageId newtype ([#167](https://github.com/nightwatch-astro/astro-up/pull/167))
- add EnumTable derive to Category for zero-alloc lookups ([#166](https://github.com/nightwatch-astro/astro-up/pull/166))
- *(004)* remove catalog.offline from CatalogConfig ([#165](https://github.com/nightwatch-astro/astro-up/pull/165))
- *(004)* move ConfigError to CoreError, add insta snapshot + proxy test
- *(003)* move error tests to integration, use CheckMethod enum

### Testing

- add missing integration tests and performance benchmarks ([#221](https://github.com/nightwatch-astro/astro-up/pull/221))

## [0.1.0](https://github.com/nightwatch-astro/astro-up/releases/tag/astro-up-core-v0.1.0) - 2026-04-05

### Bug Fixes

- use crate version in default user-agent fallback ([#764](https://github.com/nightwatch-astro/astro-up/pull/764))
- expose crate name constant for runtime identification
- add crate-level documentation to astro-up-core
- revert manual version bump, let release-plz manage versions
- bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish
- match process exe path filename to handle Linux comm truncation ([#762](https://github.com/nightwatch-astro/astro-up/pull/762))
- *(ci)* slim CI jobs, fix version-dependent snapshot ([#758](https://github.com/nightwatch-astro/astro-up/pull/758))
- delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/pull/753))
- update config snapshot for 0.1.1 version bump ([#745](https://github.com/nightwatch-astro/astro-up/pull/745))
- extract WMI data inside spawn_blocking for Send safety ([#748](https://github.com/nightwatch-astro/astro-up/pull/748))
- include minisign public key in core crate for publishing
- use NotInstalled variant for Ledger detection method ([#736](https://github.com/nightwatch-astro/astro-up/pull/736))
- handle Ledger variant in detection match arms ([#734](https://github.com/nightwatch-astro/astro-up/pull/734))
- release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/pull/732))
- skip packages with empty download URLs in orchestrator ([#728](https://github.com/nightwatch-astro/astro-up/pull/728))
- *(023)* resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/pull/729))
- PrimeVue deprecations + window size enforcement ([#655](https://github.com/nightwatch-astro/astro-up/pull/655))
- *(006)* PE detection resolves paths from install ledger ([#421](https://github.com/nightwatch-astro/astro-up/pull/421))
- *(013)* quality gates, retrospective, spec 003 reconciliation ([#420](https://github.com/nightwatch-astro/astro-up/pull/420))
- *(013)* quality gate findings, spec 003 reconciliation, pre-commit config ([#363](https://github.com/nightwatch-astro/astro-up/pull/363))
- resolve Windows clippy errors in WMI detection modules ([#260](https://github.com/nightwatch-astro/astro-up/pull/260))
- update error_display test after removing NotInstalled/UnsupportedPlatform ([#219](https://github.com/nightwatch-astro/astro-up/pull/219))
- *(ci)* inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/pull/164))
- *(003)* complete error Display test coverage, update spec FR-010
- *(003)* add missing BackupManager trait + enum validation tests

### Documentation

- add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/pull/741))

### Features

- *(ci)* add CLI integration tests on Windows ([#759](https://github.com/nightwatch-astro/astro-up/pull/759))
- *(023)* complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/pull/738))
- add Ledger detection method for firmware packages ([#731](https://github.com/nightwatch-astro/astro-up/pull/731))
- *(025)* file/directory browser picker for settings path fields ([#726](https://github.com/nightwatch-astro/astro-up/pull/726))
- *(024)* wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/pull/716))
- *(023)* lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/pull/727))
- *(022)* Vue 3 frontend — views, components, Tauri wiring ([#654](https://github.com/nightwatch-astro/astro-up/pull/654))
- *(012)* install orchestration engine ([#419](https://github.com/nightwatch-astro/astro-up/pull/419))
- *(011)* installer execution — silent install, exit codes, elevation, ZIP, uninstall ([#321](https://github.com/nightwatch-astro/astro-up/pull/321))
- *(013)* backup and restore — ZIP archives, selective restore, pruning ([#323](https://github.com/nightwatch-astro/astro-up/pull/323))
- *(010)* download manager — streaming, hash, resume, throttle, purge ([#259](https://github.com/nightwatch-astro/astro-up/pull/259))
- *(006)* software and driver detection system ([#214](https://github.com/nightwatch-astro/astro-up/pull/214))
- *(005)* manifest parsing and catalog ([#162](https://github.com/nightwatch-astro/astro-up/pull/162))
- *(004)* implement config system — load, API, tests (T011-T027)
- *(004)* implement config structs, defaults, and ConfigStore (T005-T010)
- *(004)* setup config module structure and dependencies (T001-T004)
- *(003)* T013-T043 — error types, traits, events, ledger, metrics
- *(003)* T001-T012 — workspace deps, enums, Version type
- *(001)* T004-T009, T019 — create Rust crates (core, cli, gui)

### Miscellaneous

- release v0.1.0 ([#766](https://github.com/nightwatch-astro/astro-up/pull/766))
- reset core version to 0.1.0
- disable crates.io publishing for core, use OIDC for CLI, remove CI on main push ([#763](https://github.com/nightwatch-astro/astro-up/pull/763))
- release v0.2.0 ([#760](https://github.com/nightwatch-astro/astro-up/pull/760))
- release v0.1.1 ([#742](https://github.com/nightwatch-astro/astro-up/pull/742))
- release v0.1.0 ([#163](https://github.com/nightwatch-astro/astro-up/pull/163))
- *(deps)* bump toml from 0.8.2 to 0.9.12+spec-1.1.0 ([#79](https://github.com/nightwatch-astro/astro-up/pull/79))
- *(deps)* bump strum from 0.26.3 to 0.28.0 ([#77](https://github.com/nightwatch-astro/astro-up/pull/77))

### Refactoring

- *(003)* remove superseded Detector trait and unused CoreError variants ([#218](https://github.com/nightwatch-astro/astro-up/pull/218))
- *(003)* align Software.id with PackageId newtype ([#167](https://github.com/nightwatch-astro/astro-up/pull/167))
- add EnumTable derive to Category for zero-alloc lookups ([#166](https://github.com/nightwatch-astro/astro-up/pull/166))
- *(004)* remove catalog.offline from CatalogConfig ([#165](https://github.com/nightwatch-astro/astro-up/pull/165))
- *(004)* move ConfigError to CoreError, add insta snapshot + proxy test
- *(003)* move error tests to integration, use CheckMethod enum

### Testing

- add missing integration tests and performance benchmarks ([#221](https://github.com/nightwatch-astro/astro-up/pull/221))

## [0.1.0](https://github.com/nightwatch-astro/astro-up/releases/tag/astro-up-core-v0.1.0) - 2026-04-05

### Bug Fixes

- use crate version in default user-agent fallback ([#764](https://github.com/nightwatch-astro/astro-up/pull/764))
- expose crate name constant for runtime identification
- add crate-level documentation to astro-up-core
- revert manual version bump, let release-plz manage versions
- bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish
- match process exe path filename to handle Linux comm truncation ([#762](https://github.com/nightwatch-astro/astro-up/pull/762))
- *(ci)* slim CI jobs, fix version-dependent snapshot ([#758](https://github.com/nightwatch-astro/astro-up/pull/758))
- delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/pull/753))
- update config snapshot for 0.1.1 version bump ([#745](https://github.com/nightwatch-astro/astro-up/pull/745))
- extract WMI data inside spawn_blocking for Send safety ([#748](https://github.com/nightwatch-astro/astro-up/pull/748))
- include minisign public key in core crate for publishing
- use NotInstalled variant for Ledger detection method ([#736](https://github.com/nightwatch-astro/astro-up/pull/736))
- handle Ledger variant in detection match arms ([#734](https://github.com/nightwatch-astro/astro-up/pull/734))
- release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/pull/732))
- skip packages with empty download URLs in orchestrator ([#728](https://github.com/nightwatch-astro/astro-up/pull/728))
- *(023)* resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/pull/729))
- PrimeVue deprecations + window size enforcement ([#655](https://github.com/nightwatch-astro/astro-up/pull/655))
- *(006)* PE detection resolves paths from install ledger ([#421](https://github.com/nightwatch-astro/astro-up/pull/421))
- *(013)* quality gates, retrospective, spec 003 reconciliation ([#420](https://github.com/nightwatch-astro/astro-up/pull/420))
- *(013)* quality gate findings, spec 003 reconciliation, pre-commit config ([#363](https://github.com/nightwatch-astro/astro-up/pull/363))
- resolve Windows clippy errors in WMI detection modules ([#260](https://github.com/nightwatch-astro/astro-up/pull/260))
- update error_display test after removing NotInstalled/UnsupportedPlatform ([#219](https://github.com/nightwatch-astro/astro-up/pull/219))
- *(ci)* inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/pull/164))
- *(003)* complete error Display test coverage, update spec FR-010
- *(003)* add missing BackupManager trait + enum validation tests

### Documentation

- add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/pull/741))

### Features

- *(ci)* add CLI integration tests on Windows ([#759](https://github.com/nightwatch-astro/astro-up/pull/759))
- *(023)* complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/pull/738))
- add Ledger detection method for firmware packages ([#731](https://github.com/nightwatch-astro/astro-up/pull/731))
- *(025)* file/directory browser picker for settings path fields ([#726](https://github.com/nightwatch-astro/astro-up/pull/726))
- *(024)* wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/pull/716))
- *(023)* lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/pull/727))
- *(022)* Vue 3 frontend — views, components, Tauri wiring ([#654](https://github.com/nightwatch-astro/astro-up/pull/654))
- *(012)* install orchestration engine ([#419](https://github.com/nightwatch-astro/astro-up/pull/419))
- *(011)* installer execution — silent install, exit codes, elevation, ZIP, uninstall ([#321](https://github.com/nightwatch-astro/astro-up/pull/321))
- *(013)* backup and restore — ZIP archives, selective restore, pruning ([#323](https://github.com/nightwatch-astro/astro-up/pull/323))
- *(010)* download manager — streaming, hash, resume, throttle, purge ([#259](https://github.com/nightwatch-astro/astro-up/pull/259))
- *(006)* software and driver detection system ([#214](https://github.com/nightwatch-astro/astro-up/pull/214))
- *(005)* manifest parsing and catalog ([#162](https://github.com/nightwatch-astro/astro-up/pull/162))
- *(004)* implement config system — load, API, tests (T011-T027)
- *(004)* implement config structs, defaults, and ConfigStore (T005-T010)
- *(004)* setup config module structure and dependencies (T001-T004)
- *(003)* T013-T043 — error types, traits, events, ledger, metrics
- *(003)* T001-T012 — workspace deps, enums, Version type
- *(001)* T004-T009, T019 — create Rust crates (core, cli, gui)

### Miscellaneous

- reset core version to 0.1.0
- disable crates.io publishing for core, use OIDC for CLI, remove CI on main push ([#763](https://github.com/nightwatch-astro/astro-up/pull/763))
- release v0.2.0 ([#760](https://github.com/nightwatch-astro/astro-up/pull/760))
- release v0.1.1 ([#742](https://github.com/nightwatch-astro/astro-up/pull/742))
- release v0.1.0 ([#163](https://github.com/nightwatch-astro/astro-up/pull/163))
- *(deps)* bump toml from 0.8.2 to 0.9.12+spec-1.1.0 ([#79](https://github.com/nightwatch-astro/astro-up/pull/79))
- *(deps)* bump strum from 0.26.3 to 0.28.0 ([#77](https://github.com/nightwatch-astro/astro-up/pull/77))

### Refactoring

- *(003)* remove superseded Detector trait and unused CoreError variants ([#218](https://github.com/nightwatch-astro/astro-up/pull/218))
- *(003)* align Software.id with PackageId newtype ([#167](https://github.com/nightwatch-astro/astro-up/pull/167))
- add EnumTable derive to Category for zero-alloc lookups ([#166](https://github.com/nightwatch-astro/astro-up/pull/166))
- *(004)* remove catalog.offline from CatalogConfig ([#165](https://github.com/nightwatch-astro/astro-up/pull/165))
- *(004)* move ConfigError to CoreError, add insta snapshot + proxy test
- *(003)* move error tests to integration, use CheckMethod enum

### Testing

- add missing integration tests and performance benchmarks ([#221](https://github.com/nightwatch-astro/astro-up/pull/221))

## [0.2.0](https://github.com/nightwatch-astro/astro-up/compare/astro-up-core-v0.1.1...astro-up-core-v0.2.0) - 2026-04-05

### Bug Fixes

- match process exe path filename to handle Linux comm truncation ([#762](https://github.com/nightwatch-astro/astro-up/pull/762))
- *(ci)* slim CI jobs, fix version-dependent snapshot ([#758](https://github.com/nightwatch-astro/astro-up/pull/758))
- delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/pull/753))

### Features

- *(ci)* add CLI integration tests on Windows ([#759](https://github.com/nightwatch-astro/astro-up/pull/759))
- *(023)* complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/pull/738))

## [0.1.1](https://github.com/nightwatch-astro/astro-up/compare/astro-up-core-v0.1.0...astro-up-core-v0.1.1) - 2026-04-04

### Documentation

- add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/pull/741))

## [0.1.0](https://github.com/nightwatch-astro/astro-up/releases/tag/astro-up-core-v0.1.0) - 2026-03-30

### Bug Fixes

- *(003)* complete error Display test coverage, update spec FR-010
- *(003)* add missing BackupManager trait + enum validation tests

### Features

- *(005)* manifest parsing and catalog ([#162](https://github.com/nightwatch-astro/astro-up/pull/162))
- *(004)* implement config system — load, API, tests (T011-T027)
- *(004)* implement config structs, defaults, and ConfigStore (T005-T010)
- *(004)* setup config module structure and dependencies (T001-T004)
- *(003)* T013-T043 — error types, traits, events, ledger, metrics
- *(003)* T001-T012 — workspace deps, enums, Version type
- *(001)* T004-T009, T019 — create Rust crates (core, cli, gui)

### Miscellaneous

- *(deps)* bump toml from 0.8.2 to 0.9.12+spec-1.1.0 ([#79](https://github.com/nightwatch-astro/astro-up/pull/79))
- *(deps)* bump strum from 0.26.3 to 0.28.0 ([#77](https://github.com/nightwatch-astro/astro-up/pull/77))

### Refactoring

- *(004)* move ConfigError to CoreError, add insta snapshot + proxy test
- *(003)* move error tests to integration, use CheckMethod enum
