# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.11](https://github.com/nightwatch-astro/astro-up/compare/v0.1.10...v0.1.11) (2026-04-07)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **003:** T001-T012 — workspace deps, enums, Version type ([274b6a3](https://github.com/nightwatch-astro/astro-up/commit/274b6a31d1ab4f5629113a9b55a0879e0fc16e5d))
* **003:** T013-T043 — error types, traits, events, ledger, metrics ([abf7c8a](https://github.com/nightwatch-astro/astro-up/commit/abf7c8ac1e09d65bba9ce10a04556be93edb54be))
* **004:** implement config structs, defaults, and ConfigStore (T005-T010) ([3c0b7e0](https://github.com/nightwatch-astro/astro-up/commit/3c0b7e0f90fe324d8968fb4119a5e652750de0bb))
* **004:** implement config system — load, API, tests (T011-T027) ([0b745a5](https://github.com/nightwatch-astro/astro-up/commit/0b745a5128fe0f8d4f99ebd8b90d89068e366883))
* **004:** setup config module structure and dependencies (T001-T004) ([df7e870](https://github.com/nightwatch-astro/astro-up/commit/df7e870fdfc7830c4c990d034dc3340afea55a0d))
* **004:** SQLite-backed configuration system ([#85](https://github.com/nightwatch-astro/astro-up/issues/85)) ([eb32b67](https://github.com/nightwatch-astro/astro-up/commit/eb32b67b0f2eb030925ff028889e38c656aad1ba))
* **005:** manifest parsing and catalog ([#162](https://github.com/nightwatch-astro/astro-up/issues/162)) ([88e396b](https://github.com/nightwatch-astro/astro-up/commit/88e396b4be8f99f424c96e7ac3e357b587d133e3))
* **006:** software and driver detection system ([#214](https://github.com/nightwatch-astro/astro-up/issues/214)) ([f57999a](https://github.com/nightwatch-astro/astro-up/commit/f57999a9c8b921045bdf2ac803bd23c2467cb13c))
* **010:** download manager — streaming, hash, resume, throttle, purge ([#259](https://github.com/nightwatch-astro/astro-up/issues/259)) ([7c41093](https://github.com/nightwatch-astro/astro-up/commit/7c41093d6dad9e9fa0b8abb5e5f72b61bdff47db))
* **011:** installer execution — silent install, exit codes, elevation, ZIP, uninstall ([#321](https://github.com/nightwatch-astro/astro-up/issues/321)) ([209a483](https://github.com/nightwatch-astro/astro-up/commit/209a4837df7847d2dc6a12c407101fb90e1817e9))
* **012:** install orchestration engine ([#419](https://github.com/nightwatch-astro/astro-up/issues/419)) ([8aec870](https://github.com/nightwatch-astro/astro-up/commit/8aec870e2ac02474b2466a70415df9dca0a0367e))
* **013:** backup and restore — ZIP archives, selective restore, pruning ([#323](https://github.com/nightwatch-astro/astro-up/issues/323)) ([c2d1c67](https://github.com/nightwatch-astro/astro-up/commit/c2d1c675768f43a29217280bf8e86cec857431c5))
* **022:** Vue 3 frontend — views, components, Tauri wiring ([#654](https://github.com/nightwatch-astro/astro-up/issues/654)) ([13eade7](https://github.com/nightwatch-astro/astro-up/commit/13eade713e6b64e86e505b8d4e6beced00dd3971))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **023:** lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/issues/727)) ([b2e6709](https://github.com/nightwatch-astro/astro-up/commit/b2e670953dae6bb780c571643399f49beca1f1b6))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* **025:** file/directory browser picker for settings path fields ([#726](https://github.com/nightwatch-astro/astro-up/issues/726)) ([aa276e0](https://github.com/nightwatch-astro/astro-up/commit/aa276e0f1fa5962b8c44753770dbb9fbce18dcc5))
* add Ledger detection method for firmware packages ([#731](https://github.com/nightwatch-astro/astro-up/issues/731)) ([61cec39](https://github.com/nightwatch-astro/astro-up/commit/61cec3977b44171bc950e68e42690c84566f6f50))
* **ci:** add CLI integration tests on Windows ([#759](https://github.com/nightwatch-astro/astro-up/issues/759)) ([6e1b4f4](https://github.com/nightwatch-astro/astro-up/commit/6e1b4f4f271e5b0c3678c25f1dc0da0c70059b3a))
* **detect:** WMI-based detection replaces per-package registry lookups ([#827](https://github.com/nightwatch-astro/astro-up/issues/827)) ([6651da8](https://github.com/nightwatch-astro/astro-up/commit/6651da827856c7fbcab21f7e3ed5da3b65b3ff59))
* **download:** resolve download URL from release assets ([#829](https://github.com/nightwatch-astro/astro-up/issues/829)) ([ffa9e20](https://github.com/nightwatch-astro/astro-up/commit/ffa9e20c003412c6bbcb627f3cd049334b52fbc5))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 003 — core domain types ([5a7f7bc](https://github.com/nightwatch-astro/astro-up/commit/5a7f7bc89991a5d26a55c6ab8c94a45c41b6c187))


### Bug Fixes

* **003:** add missing BackupManager trait + enum validation tests ([fb2f3e1](https://github.com/nightwatch-astro/astro-up/commit/fb2f3e1ea63f313f5d934c7f37b30a8eaa368f8f))
* **003:** complete error Display test coverage, update spec FR-010 ([79669ab](https://github.com/nightwatch-astro/astro-up/commit/79669ab913fdebb85ceebc18da15281e42cc1c82))
* **006:** PE detection resolves paths from install ledger ([#421](https://github.com/nightwatch-astro/astro-up/issues/421)) ([abb51c2](https://github.com/nightwatch-astro/astro-up/commit/abb51c26097ea6a829dd580bd07b572f7bf14a20))
* **011:** address verify findings — timeout Duration, contract alignment, zip-slip test ([e434e5d](https://github.com/nightwatch-astro/astro-up/commit/e434e5d93b420fadbc1c6c166814f5723708d087))
* **011:** verify findings and cleanup ([0dbaa70](https://github.com/nightwatch-astro/astro-up/commit/0dbaa7008b886fa42b435bcb214145d88bb2e635))
* **013:** quality gate findings, spec 003 reconciliation, pre-commit config ([#363](https://github.com/nightwatch-astro/astro-up/issues/363)) ([28d002a](https://github.com/nightwatch-astro/astro-up/commit/28d002ada53d6ec46ec44b4ed83c579b04a812ee))
* **013:** quality gates, retrospective, spec 003 reconciliation ([#420](https://github.com/nightwatch-astro/astro-up/issues/420)) ([e5d985a](https://github.com/nightwatch-astro/astro-up/commit/e5d985ae9f584943d77b19a570febdb1b9de14ef))
* **023:** resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/issues/729)) ([a956f4a](https://github.com/nightwatch-astro/astro-up/commit/a956f4a9585f120df91c936431054114c1c8f540))
* add crate-level documentation to astro-up-core ([054033b](https://github.com/nightwatch-astro/astro-up/commit/054033bdc0a7d40544a680ff681d8907d0e8f81d))
* bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish ([b5b4dc9](https://github.com/nightwatch-astro/astro-up/commit/b5b4dc971804756d4eae846206f520d1254cb912))
* **catalog:** read install config from catalog, drop old detection schema ([#828](https://github.com/nightwatch-astro/astro-up/issues/828)) ([80ece7d](https://github.com/nightwatch-astro/astro-up/commit/80ece7d98a2095f24e7ec9022f1c5f3a43146321))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* **ci:** slim CI jobs, fix version-dependent snapshot ([#758](https://github.com/nightwatch-astro/astro-up/issues/758)) ([f9a8a68](https://github.com/nightwatch-astro/astro-up/commit/f9a8a684be040885241f805843ad45c459b65619))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* expose crate name constant for runtime identification ([7ffbf7e](https://github.com/nightwatch-astro/astro-up/commit/7ffbf7e50a14439ba8ed58358ce4fd6116c66ae8))
* extract WMI data inside spawn_blocking for Send safety ([#748](https://github.com/nightwatch-astro/astro-up/issues/748)) ([c9e2bef](https://github.com/nightwatch-astro/astro-up/commit/c9e2bef0dedf82f67bafe5e5170c7c586a436fed))
* handle Ledger variant in detection match arms ([#734](https://github.com/nightwatch-astro/astro-up/issues/734)) ([73af7ba](https://github.com/nightwatch-astro/astro-up/commit/73af7baab2d9c88f5d8ef4d36944ca3ecb7dd3d8))
* include minisign public key in core crate for publishing ([9ee4b14](https://github.com/nightwatch-astro/astro-up/commit/9ee4b14b744fe382ea0f5c790aa8538f2b1449b9))
* **install:** catch OS error 740 at process spawn for elevation ([#826](https://github.com/nightwatch-astro/astro-up/issues/826)) ([82faf50](https://github.com/nightwatch-astro/astro-up/commit/82faf50f3e314e299570f669cd8e894a517c2afa))
* match process exe path filename to handle Linux comm truncation ([#762](https://github.com/nightwatch-astro/astro-up/issues/762)) ([f6c851f](https://github.com/nightwatch-astro/astro-up/commit/f6c851f744b945aa806ca3a942a95431ce03f57d))
* PrimeVue deprecations + window size enforcement ([#655](https://github.com/nightwatch-astro/astro-up/issues/655)) ([6d4a72e](https://github.com/nightwatch-astro/astro-up/commit/6d4a72ec978ca2c6eb7e17ec5f39efc0f518f62f))
* release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/issues/732)) ([147c225](https://github.com/nightwatch-astro/astro-up/commit/147c225ef8b5aa474f71dc38eba4dd06c483989a))
* **release:** sync core version to 0.1.10 and add project name to release title ([#858](https://github.com/nightwatch-astro/astro-up/issues/858)) ([f2aed04](https://github.com/nightwatch-astro/astro-up/commit/f2aed04d22d141bb9ea72f4a766a2c22a6b4e5fa))
* resolve Windows clippy errors in WMI detection modules ([#260](https://github.com/nightwatch-astro/astro-up/issues/260)) ([4745e73](https://github.com/nightwatch-astro/astro-up/commit/4745e73b80b12279c9cf82db5411e12e9f8f06d9))
* revert manual version bump, let release-plz manage versions ([c0dab2d](https://github.com/nightwatch-astro/astro-up/commit/c0dab2dbaa4d619500cef7a4a560ffcd0ce319b5))
* skip packages with empty download URLs in orchestrator ([#728](https://github.com/nightwatch-astro/astro-up/issues/728)) ([dbaa504](https://github.com/nightwatch-astro/astro-up/commit/dbaa5040916a900767600efffa592b5c456a7291))
* update config snapshot for 0.1.1 version bump ([#745](https://github.com/nightwatch-astro/astro-up/issues/745)) ([eeb12df](https://github.com/nightwatch-astro/astro-up/commit/eeb12dfb100e67e10b38acebd53fa01fb98ded2e))
* update error_display test after removing NotInstalled/UnsupportedPlatform ([#219](https://github.com/nightwatch-astro/astro-up/issues/219)) ([d7f97d1](https://github.com/nightwatch-astro/astro-up/commit/d7f97d1a6ae98cad329dea6ac85df8f93614e7b0))
* use crate version in default user-agent fallback ([#764](https://github.com/nightwatch-astro/astro-up/issues/764)) ([dbb091e](https://github.com/nightwatch-astro/astro-up/commit/dbb091ebefcb51c2dde029e9b871d2180518d6d2))
* use NotInstalled variant for Ledger detection method ([#736](https://github.com/nightwatch-astro/astro-up/issues/736)) ([b0d3cc9](https://github.com/nightwatch-astro/astro-up/commit/b0d3cc9083e7df259fca11f626577ea9171c39a1))


### Refactoring

* **003:** align Software.id with PackageId newtype ([#167](https://github.com/nightwatch-astro/astro-up/issues/167)) ([ab7068e](https://github.com/nightwatch-astro/astro-up/commit/ab7068e41c4265393c415b6dfd7a01cd4d1f651e)), closes [#161](https://github.com/nightwatch-astro/astro-up/issues/161)
* **003:** move error tests to integration, use CheckMethod enum ([1fd8630](https://github.com/nightwatch-astro/astro-up/commit/1fd86302e6335a631b7c97b9ba9a347a4901af1c))
* **003:** remove superseded Detector trait and unused CoreError variants ([#218](https://github.com/nightwatch-astro/astro-up/issues/218)) ([eda1d62](https://github.com/nightwatch-astro/astro-up/commit/eda1d625d44e10ddae60142df8a230afcaa5cf5a)), closes [#216](https://github.com/nightwatch-astro/astro-up/issues/216)
* **004:** move ConfigError to CoreError, add insta snapshot + proxy test ([835e824](https://github.com/nightwatch-astro/astro-up/commit/835e824de7b665edfb5eab1baeb03d35f62e6f80))
* **004:** remove catalog.offline from CatalogConfig ([#165](https://github.com/nightwatch-astro/astro-up/issues/165)) ([1be2fed](https://github.com/nightwatch-astro/astro-up/commit/1be2fed9ce9e794341012c5c623e4c476f9ca034)), closes [#118](https://github.com/nightwatch-astro/astro-up/issues/118)
* **011:** cleanup — deduplicate hooks, fix TOCTOU, idiomatic types ([909a6bc](https://github.com/nightwatch-astro/astro-up/commit/909a6bc7c9af600601707783f8f308c5aaafbebd))
* add EnumTable derive to Category for zero-alloc lookups ([#166](https://github.com/nightwatch-astro/astro-up/issues/166)) ([d6d2445](https://github.com/nightwatch-astro/astro-up/commit/d6d2445be1a4b41c753ab254b626cbe19fc85e84)), closes [#80](https://github.com/nightwatch-astro/astro-up/issues/80)


### Documentation

* add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/issues/741)) ([332bd6a](https://github.com/nightwatch-astro/astro-up/commit/332bd6a258bf79eb331d20093f8b28041418d2dc))


### Testing

* **015:** deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/issues/462)) ([63f5cd5](https://github.com/nightwatch-astro/astro-up/commit/63f5cd55ddc6020056224bdbad822202d16f2939))
* add missing integration tests and performance benchmarks ([#221](https://github.com/nightwatch-astro/astro-up/issues/221)) ([a0c5df3](https://github.com/nightwatch-astro/astro-up/commit/a0c5df35b2eba06e8472bcd8213059a09fa3f8cb))

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
