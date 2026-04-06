# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.8](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.7...astro-up-cli-v0.1.8) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **015:** CLI interface — commands, output modes, logging ([9b13e02](https://github.com/nightwatch-astro/astro-up/commit/9b13e026e66ed78e505eb72674a8225d22820066))
* **015:** CLI scaffold — deps, modules, dispatch, logging (T001-T011) ([d1b6c87](https://github.com/nightwatch-astro/astro-up/commit/d1b6c879a0b38356ec02efd1d2b8410776156456))
* **015:** install, update, backup, restore, config, self-update, exit codes (T021-T035) ([9870961](https://github.com/nightwatch-astro/astro-up/commit/9870961dcf6413cb50337221b1e40bcee0936c2e))
* **015:** scan and search commands (T017-T020) ([93b0269](https://github.com/nightwatch-astro/astro-up/commit/93b02699b7e7391c61430b91adf1be05422b361c))
* **015:** show command — catalog listing, detail, backups (T012-T016) ([dee0ea6](https://github.com/nightwatch-astro/astro-up/commit/dee0ea6aa799ee5aaab7728b36f73bcfd555f1ff))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **023:** lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/issues/727)) ([b2e6709](https://github.com/nightwatch-astro/astro-up/commit/b2e670953dae6bb780c571643399f49beca1f1b6))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/issues/656)) ([28855fc](https://github.com/nightwatch-astro/astro-up/commit/28855fc22f44422129c8685b8b275cec89eae862))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **015:** exit code 2 for cancellation, --quiet suppresses tracing, log dir path ([25ceb51](https://github.com/nightwatch-astro/astro-up/commit/25ceb5120584ced06cabc86ecf103f6dd717e465))
* **023:** resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/issues/729)) ([a956f4a](https://github.com/nightwatch-astro/astro-up/commit/a956f4a9585f120df91c936431054114c1c8f540))
* avoid catalog lock contention in global_json_flag_accepted test ([#801](https://github.com/nightwatch-astro/astro-up/issues/801)) ([21a146d](https://github.com/nightwatch-astro/astro-up/commit/21a146d4f26635341060c15b2385a6b4ffff9579))
* bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish ([b5b4dc9](https://github.com/nightwatch-astro/astro-up/commit/b5b4dc971804756d4eae846206f520d1254cb912))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* **cli:** rewrite self-update to use self_update crate ([#769](https://github.com/nightwatch-astro/astro-up/issues/769)) ([7250835](https://github.com/nightwatch-astro/astro-up/commit/72508357c5ab01a1289b30181f1d48978c2a9edc))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* ignore json_update_valid test (hangs on CI) ([#799](https://github.com/nightwatch-astro/astro-up/issues/799)) ([c02aa8e](https://github.com/nightwatch-astro/astro-up/commit/c02aa8e10ccdbcc8308f57ffbac17df412b3ec37))
* log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/issues/765)) ([bafc4e6](https://github.com/nightwatch-astro/astro-up/commit/bafc4e6dfdf1a541736808c8c24ad425531ed829))
* release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/issues/732)) ([147c225](https://github.com/nightwatch-astro/astro-up/commit/147c225ef8b5aa474f71dc38eba4dd06c483989a))
* revert manual version bump, let release-plz manage versions ([c0dab2d](https://github.com/nightwatch-astro/astro-up/commit/c0dab2dbaa4d619500cef7a4a560ffcd0ce319b5))
* rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/issues/767)) ([75c14ef](https://github.com/nightwatch-astro/astro-up/commit/75c14efc12620cc8068b89ce229b3b50123ef32f))
* skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/issues/746)) ([ed75b43](https://github.com/nightwatch-astro/astro-up/commit/ed75b434a430f63899b694e26b209499cb9c20b0))


### Documentation

* add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/issues/741)) ([332bd6a](https://github.com/nightwatch-astro/astro-up/commit/332bd6a258bf79eb331d20093f8b28041418d2dc))


### Testing

* **015:** deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/issues/462)) ([63f5cd5](https://github.com/nightwatch-astro/astro-up/commit/63f5cd55ddc6020056224bdbad822202d16f2939))

## [0.1.7](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.6...astro-up-cli-v0.1.7) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **015:** CLI interface — commands, output modes, logging ([9b13e02](https://github.com/nightwatch-astro/astro-up/commit/9b13e026e66ed78e505eb72674a8225d22820066))
* **015:** CLI scaffold — deps, modules, dispatch, logging (T001-T011) ([d1b6c87](https://github.com/nightwatch-astro/astro-up/commit/d1b6c879a0b38356ec02efd1d2b8410776156456))
* **015:** install, update, backup, restore, config, self-update, exit codes (T021-T035) ([9870961](https://github.com/nightwatch-astro/astro-up/commit/9870961dcf6413cb50337221b1e40bcee0936c2e))
* **015:** scan and search commands (T017-T020) ([93b0269](https://github.com/nightwatch-astro/astro-up/commit/93b02699b7e7391c61430b91adf1be05422b361c))
* **015:** show command — catalog listing, detail, backups (T012-T016) ([dee0ea6](https://github.com/nightwatch-astro/astro-up/commit/dee0ea6aa799ee5aaab7728b36f73bcfd555f1ff))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **023:** lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/issues/727)) ([b2e6709](https://github.com/nightwatch-astro/astro-up/commit/b2e670953dae6bb780c571643399f49beca1f1b6))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/issues/656)) ([28855fc](https://github.com/nightwatch-astro/astro-up/commit/28855fc22f44422129c8685b8b275cec89eae862))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **015:** exit code 2 for cancellation, --quiet suppresses tracing, log dir path ([25ceb51](https://github.com/nightwatch-astro/astro-up/commit/25ceb5120584ced06cabc86ecf103f6dd717e465))
* **023:** resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/issues/729)) ([a956f4a](https://github.com/nightwatch-astro/astro-up/commit/a956f4a9585f120df91c936431054114c1c8f540))
* avoid catalog lock contention in global_json_flag_accepted test ([#801](https://github.com/nightwatch-astro/astro-up/issues/801)) ([21a146d](https://github.com/nightwatch-astro/astro-up/commit/21a146d4f26635341060c15b2385a6b4ffff9579))
* bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish ([b5b4dc9](https://github.com/nightwatch-astro/astro-up/commit/b5b4dc971804756d4eae846206f520d1254cb912))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* **cli:** rewrite self-update to use self_update crate ([#769](https://github.com/nightwatch-astro/astro-up/issues/769)) ([7250835](https://github.com/nightwatch-astro/astro-up/commit/72508357c5ab01a1289b30181f1d48978c2a9edc))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* ignore json_update_valid test (hangs on CI) ([#799](https://github.com/nightwatch-astro/astro-up/issues/799)) ([c02aa8e](https://github.com/nightwatch-astro/astro-up/commit/c02aa8e10ccdbcc8308f57ffbac17df412b3ec37))
* log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/issues/765)) ([bafc4e6](https://github.com/nightwatch-astro/astro-up/commit/bafc4e6dfdf1a541736808c8c24ad425531ed829))
* release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/issues/732)) ([147c225](https://github.com/nightwatch-astro/astro-up/commit/147c225ef8b5aa474f71dc38eba4dd06c483989a))
* revert manual version bump, let release-plz manage versions ([c0dab2d](https://github.com/nightwatch-astro/astro-up/commit/c0dab2dbaa4d619500cef7a4a560ffcd0ce319b5))
* rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/issues/767)) ([75c14ef](https://github.com/nightwatch-astro/astro-up/commit/75c14efc12620cc8068b89ce229b3b50123ef32f))
* skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/issues/746)) ([ed75b43](https://github.com/nightwatch-astro/astro-up/commit/ed75b434a430f63899b694e26b209499cb9c20b0))


### Documentation

* add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/issues/741)) ([332bd6a](https://github.com/nightwatch-astro/astro-up/commit/332bd6a258bf79eb331d20093f8b28041418d2dc))


### Testing

* **015:** deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/issues/462)) ([63f5cd5](https://github.com/nightwatch-astro/astro-up/commit/63f5cd55ddc6020056224bdbad822202d16f2939))

## [0.1.6](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.5...astro-up-cli-v0.1.6) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **015:** CLI interface — commands, output modes, logging ([9b13e02](https://github.com/nightwatch-astro/astro-up/commit/9b13e026e66ed78e505eb72674a8225d22820066))
* **015:** CLI scaffold — deps, modules, dispatch, logging (T001-T011) ([d1b6c87](https://github.com/nightwatch-astro/astro-up/commit/d1b6c879a0b38356ec02efd1d2b8410776156456))
* **015:** install, update, backup, restore, config, self-update, exit codes (T021-T035) ([9870961](https://github.com/nightwatch-astro/astro-up/commit/9870961dcf6413cb50337221b1e40bcee0936c2e))
* **015:** scan and search commands (T017-T020) ([93b0269](https://github.com/nightwatch-astro/astro-up/commit/93b02699b7e7391c61430b91adf1be05422b361c))
* **015:** show command — catalog listing, detail, backups (T012-T016) ([dee0ea6](https://github.com/nightwatch-astro/astro-up/commit/dee0ea6aa799ee5aaab7728b36f73bcfd555f1ff))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **023:** lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/issues/727)) ([b2e6709](https://github.com/nightwatch-astro/astro-up/commit/b2e670953dae6bb780c571643399f49beca1f1b6))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/issues/656)) ([28855fc](https://github.com/nightwatch-astro/astro-up/commit/28855fc22f44422129c8685b8b275cec89eae862))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **015:** exit code 2 for cancellation, --quiet suppresses tracing, log dir path ([25ceb51](https://github.com/nightwatch-astro/astro-up/commit/25ceb5120584ced06cabc86ecf103f6dd717e465))
* **023:** resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/issues/729)) ([a956f4a](https://github.com/nightwatch-astro/astro-up/commit/a956f4a9585f120df91c936431054114c1c8f540))
* avoid catalog lock contention in global_json_flag_accepted test ([#801](https://github.com/nightwatch-astro/astro-up/issues/801)) ([21a146d](https://github.com/nightwatch-astro/astro-up/commit/21a146d4f26635341060c15b2385a6b4ffff9579))
* bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish ([b5b4dc9](https://github.com/nightwatch-astro/astro-up/commit/b5b4dc971804756d4eae846206f520d1254cb912))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* **cli:** rewrite self-update to use self_update crate ([#769](https://github.com/nightwatch-astro/astro-up/issues/769)) ([7250835](https://github.com/nightwatch-astro/astro-up/commit/72508357c5ab01a1289b30181f1d48978c2a9edc))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* ignore json_update_valid test (hangs on CI) ([#799](https://github.com/nightwatch-astro/astro-up/issues/799)) ([c02aa8e](https://github.com/nightwatch-astro/astro-up/commit/c02aa8e10ccdbcc8308f57ffbac17df412b3ec37))
* log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/issues/765)) ([bafc4e6](https://github.com/nightwatch-astro/astro-up/commit/bafc4e6dfdf1a541736808c8c24ad425531ed829))
* release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/issues/732)) ([147c225](https://github.com/nightwatch-astro/astro-up/commit/147c225ef8b5aa474f71dc38eba4dd06c483989a))
* revert manual version bump, let release-plz manage versions ([c0dab2d](https://github.com/nightwatch-astro/astro-up/commit/c0dab2dbaa4d619500cef7a4a560ffcd0ce319b5))
* rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/issues/767)) ([75c14ef](https://github.com/nightwatch-astro/astro-up/commit/75c14efc12620cc8068b89ce229b3b50123ef32f))
* skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/issues/746)) ([ed75b43](https://github.com/nightwatch-astro/astro-up/commit/ed75b434a430f63899b694e26b209499cb9c20b0))


### Documentation

* add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/issues/741)) ([332bd6a](https://github.com/nightwatch-astro/astro-up/commit/332bd6a258bf79eb331d20093f8b28041418d2dc))


### Testing

* **015:** deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/issues/462)) ([63f5cd5](https://github.com/nightwatch-astro/astro-up/commit/63f5cd55ddc6020056224bdbad822202d16f2939))

## [0.1.5](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.4...astro-up-cli-v0.1.5) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **015:** CLI interface — commands, output modes, logging ([9b13e02](https://github.com/nightwatch-astro/astro-up/commit/9b13e026e66ed78e505eb72674a8225d22820066))
* **015:** CLI scaffold — deps, modules, dispatch, logging (T001-T011) ([d1b6c87](https://github.com/nightwatch-astro/astro-up/commit/d1b6c879a0b38356ec02efd1d2b8410776156456))
* **015:** install, update, backup, restore, config, self-update, exit codes (T021-T035) ([9870961](https://github.com/nightwatch-astro/astro-up/commit/9870961dcf6413cb50337221b1e40bcee0936c2e))
* **015:** scan and search commands (T017-T020) ([93b0269](https://github.com/nightwatch-astro/astro-up/commit/93b02699b7e7391c61430b91adf1be05422b361c))
* **015:** show command — catalog listing, detail, backups (T012-T016) ([dee0ea6](https://github.com/nightwatch-astro/astro-up/commit/dee0ea6aa799ee5aaab7728b36f73bcfd555f1ff))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **023:** lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/issues/727)) ([b2e6709](https://github.com/nightwatch-astro/astro-up/commit/b2e670953dae6bb780c571643399f49beca1f1b6))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/issues/656)) ([28855fc](https://github.com/nightwatch-astro/astro-up/commit/28855fc22f44422129c8685b8b275cec89eae862))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **015:** exit code 2 for cancellation, --quiet suppresses tracing, log dir path ([25ceb51](https://github.com/nightwatch-astro/astro-up/commit/25ceb5120584ced06cabc86ecf103f6dd717e465))
* **023:** resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/issues/729)) ([a956f4a](https://github.com/nightwatch-astro/astro-up/commit/a956f4a9585f120df91c936431054114c1c8f540))
* avoid catalog lock contention in global_json_flag_accepted test ([#801](https://github.com/nightwatch-astro/astro-up/issues/801)) ([21a146d](https://github.com/nightwatch-astro/astro-up/commit/21a146d4f26635341060c15b2385a6b4ffff9579))
* bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish ([b5b4dc9](https://github.com/nightwatch-astro/astro-up/commit/b5b4dc971804756d4eae846206f520d1254cb912))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* **cli:** rewrite self-update to use self_update crate ([#769](https://github.com/nightwatch-astro/astro-up/issues/769)) ([7250835](https://github.com/nightwatch-astro/astro-up/commit/72508357c5ab01a1289b30181f1d48978c2a9edc))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* ignore json_update_valid test (hangs on CI) ([#799](https://github.com/nightwatch-astro/astro-up/issues/799)) ([c02aa8e](https://github.com/nightwatch-astro/astro-up/commit/c02aa8e10ccdbcc8308f57ffbac17df412b3ec37))
* log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/issues/765)) ([bafc4e6](https://github.com/nightwatch-astro/astro-up/commit/bafc4e6dfdf1a541736808c8c24ad425531ed829))
* release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/issues/732)) ([147c225](https://github.com/nightwatch-astro/astro-up/commit/147c225ef8b5aa474f71dc38eba4dd06c483989a))
* revert manual version bump, let release-plz manage versions ([c0dab2d](https://github.com/nightwatch-astro/astro-up/commit/c0dab2dbaa4d619500cef7a4a560ffcd0ce319b5))
* rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/issues/767)) ([75c14ef](https://github.com/nightwatch-astro/astro-up/commit/75c14efc12620cc8068b89ce229b3b50123ef32f))
* skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/issues/746)) ([ed75b43](https://github.com/nightwatch-astro/astro-up/commit/ed75b434a430f63899b694e26b209499cb9c20b0))


### Documentation

* add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/issues/741)) ([332bd6a](https://github.com/nightwatch-astro/astro-up/commit/332bd6a258bf79eb331d20093f8b28041418d2dc))


### Testing

* **015:** deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/issues/462)) ([63f5cd5](https://github.com/nightwatch-astro/astro-up/commit/63f5cd55ddc6020056224bdbad822202d16f2939))

## [0.1.4](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.3...astro-up-cli-v0.1.4) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **015:** CLI interface — commands, output modes, logging ([9b13e02](https://github.com/nightwatch-astro/astro-up/commit/9b13e026e66ed78e505eb72674a8225d22820066))
* **015:** CLI scaffold — deps, modules, dispatch, logging (T001-T011) ([d1b6c87](https://github.com/nightwatch-astro/astro-up/commit/d1b6c879a0b38356ec02efd1d2b8410776156456))
* **015:** install, update, backup, restore, config, self-update, exit codes (T021-T035) ([9870961](https://github.com/nightwatch-astro/astro-up/commit/9870961dcf6413cb50337221b1e40bcee0936c2e))
* **015:** scan and search commands (T017-T020) ([93b0269](https://github.com/nightwatch-astro/astro-up/commit/93b02699b7e7391c61430b91adf1be05422b361c))
* **015:** show command — catalog listing, detail, backups (T012-T016) ([dee0ea6](https://github.com/nightwatch-astro/astro-up/commit/dee0ea6aa799ee5aaab7728b36f73bcfd555f1ff))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **023:** lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/issues/727)) ([b2e6709](https://github.com/nightwatch-astro/astro-up/commit/b2e670953dae6bb780c571643399f49beca1f1b6))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/issues/656)) ([28855fc](https://github.com/nightwatch-astro/astro-up/commit/28855fc22f44422129c8685b8b275cec89eae862))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **015:** exit code 2 for cancellation, --quiet suppresses tracing, log dir path ([25ceb51](https://github.com/nightwatch-astro/astro-up/commit/25ceb5120584ced06cabc86ecf103f6dd717e465))
* **023:** resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/issues/729)) ([a956f4a](https://github.com/nightwatch-astro/astro-up/commit/a956f4a9585f120df91c936431054114c1c8f540))
* avoid catalog lock contention in global_json_flag_accepted test ([#801](https://github.com/nightwatch-astro/astro-up/issues/801)) ([21a146d](https://github.com/nightwatch-astro/astro-up/commit/21a146d4f26635341060c15b2385a6b4ffff9579))
* bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish ([b5b4dc9](https://github.com/nightwatch-astro/astro-up/commit/b5b4dc971804756d4eae846206f520d1254cb912))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* **cli:** rewrite self-update to use self_update crate ([#769](https://github.com/nightwatch-astro/astro-up/issues/769)) ([7250835](https://github.com/nightwatch-astro/astro-up/commit/72508357c5ab01a1289b30181f1d48978c2a9edc))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* ignore json_update_valid test (hangs on CI) ([#799](https://github.com/nightwatch-astro/astro-up/issues/799)) ([c02aa8e](https://github.com/nightwatch-astro/astro-up/commit/c02aa8e10ccdbcc8308f57ffbac17df412b3ec37))
* log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/issues/765)) ([bafc4e6](https://github.com/nightwatch-astro/astro-up/commit/bafc4e6dfdf1a541736808c8c24ad425531ed829))
* release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/issues/732)) ([147c225](https://github.com/nightwatch-astro/astro-up/commit/147c225ef8b5aa474f71dc38eba4dd06c483989a))
* revert manual version bump, let release-plz manage versions ([c0dab2d](https://github.com/nightwatch-astro/astro-up/commit/c0dab2dbaa4d619500cef7a4a560ffcd0ce319b5))
* rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/issues/767)) ([75c14ef](https://github.com/nightwatch-astro/astro-up/commit/75c14efc12620cc8068b89ce229b3b50123ef32f))
* skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/issues/746)) ([ed75b43](https://github.com/nightwatch-astro/astro-up/commit/ed75b434a430f63899b694e26b209499cb9c20b0))


### Documentation

* add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/issues/741)) ([332bd6a](https://github.com/nightwatch-astro/astro-up/commit/332bd6a258bf79eb331d20093f8b28041418d2dc))


### Testing

* **015:** deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/issues/462)) ([63f5cd5](https://github.com/nightwatch-astro/astro-up/commit/63f5cd55ddc6020056224bdbad822202d16f2939))

## [0.1.3](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.2...astro-up-cli-v0.1.3) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **015:** CLI interface — commands, output modes, logging ([9b13e02](https://github.com/nightwatch-astro/astro-up/commit/9b13e026e66ed78e505eb72674a8225d22820066))
* **015:** CLI scaffold — deps, modules, dispatch, logging (T001-T011) ([d1b6c87](https://github.com/nightwatch-astro/astro-up/commit/d1b6c879a0b38356ec02efd1d2b8410776156456))
* **015:** install, update, backup, restore, config, self-update, exit codes (T021-T035) ([9870961](https://github.com/nightwatch-astro/astro-up/commit/9870961dcf6413cb50337221b1e40bcee0936c2e))
* **015:** scan and search commands (T017-T020) ([93b0269](https://github.com/nightwatch-astro/astro-up/commit/93b02699b7e7391c61430b91adf1be05422b361c))
* **015:** show command — catalog listing, detail, backups (T012-T016) ([dee0ea6](https://github.com/nightwatch-astro/astro-up/commit/dee0ea6aa799ee5aaab7728b36f73bcfd555f1ff))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **023:** lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/issues/727)) ([b2e6709](https://github.com/nightwatch-astro/astro-up/commit/b2e670953dae6bb780c571643399f49beca1f1b6))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/issues/656)) ([28855fc](https://github.com/nightwatch-astro/astro-up/commit/28855fc22f44422129c8685b8b275cec89eae862))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **015:** exit code 2 for cancellation, --quiet suppresses tracing, log dir path ([25ceb51](https://github.com/nightwatch-astro/astro-up/commit/25ceb5120584ced06cabc86ecf103f6dd717e465))
* **023:** resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/issues/729)) ([a956f4a](https://github.com/nightwatch-astro/astro-up/commit/a956f4a9585f120df91c936431054114c1c8f540))
* avoid catalog lock contention in global_json_flag_accepted test ([#801](https://github.com/nightwatch-astro/astro-up/issues/801)) ([21a146d](https://github.com/nightwatch-astro/astro-up/commit/21a146d4f26635341060c15b2385a6b4ffff9579))
* bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish ([b5b4dc9](https://github.com/nightwatch-astro/astro-up/commit/b5b4dc971804756d4eae846206f520d1254cb912))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* **cli:** rewrite self-update to use self_update crate ([#769](https://github.com/nightwatch-astro/astro-up/issues/769)) ([7250835](https://github.com/nightwatch-astro/astro-up/commit/72508357c5ab01a1289b30181f1d48978c2a9edc))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* ignore json_update_valid test (hangs on CI) ([#799](https://github.com/nightwatch-astro/astro-up/issues/799)) ([c02aa8e](https://github.com/nightwatch-astro/astro-up/commit/c02aa8e10ccdbcc8308f57ffbac17df412b3ec37))
* log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/issues/765)) ([bafc4e6](https://github.com/nightwatch-astro/astro-up/commit/bafc4e6dfdf1a541736808c8c24ad425531ed829))
* release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/issues/732)) ([147c225](https://github.com/nightwatch-astro/astro-up/commit/147c225ef8b5aa474f71dc38eba4dd06c483989a))
* revert manual version bump, let release-plz manage versions ([c0dab2d](https://github.com/nightwatch-astro/astro-up/commit/c0dab2dbaa4d619500cef7a4a560ffcd0ce319b5))
* rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/issues/767)) ([75c14ef](https://github.com/nightwatch-astro/astro-up/commit/75c14efc12620cc8068b89ce229b3b50123ef32f))
* skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/issues/746)) ([ed75b43](https://github.com/nightwatch-astro/astro-up/commit/ed75b434a430f63899b694e26b209499cb9c20b0))


### Documentation

* add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/issues/741)) ([332bd6a](https://github.com/nightwatch-astro/astro-up/commit/332bd6a258bf79eb331d20093f8b28041418d2dc))


### Testing

* **015:** deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/issues/462)) ([63f5cd5](https://github.com/nightwatch-astro/astro-up/commit/63f5cd55ddc6020056224bdbad822202d16f2939))

## [0.1.2](https://github.com/nightwatch-astro/astro-up/compare/astro-up-cli-v0.1.1...astro-up-cli-v0.1.2) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **015:** CLI interface — commands, output modes, logging ([9b13e02](https://github.com/nightwatch-astro/astro-up/commit/9b13e026e66ed78e505eb72674a8225d22820066))
* **015:** CLI scaffold — deps, modules, dispatch, logging (T001-T011) ([d1b6c87](https://github.com/nightwatch-astro/astro-up/commit/d1b6c879a0b38356ec02efd1d2b8410776156456))
* **015:** install, update, backup, restore, config, self-update, exit codes (T021-T035) ([9870961](https://github.com/nightwatch-astro/astro-up/commit/9870961dcf6413cb50337221b1e40bcee0936c2e))
* **015:** scan and search commands (T017-T020) ([93b0269](https://github.com/nightwatch-astro/astro-up/commit/93b02699b7e7391c61430b91adf1be05422b361c))
* **015:** show command — catalog listing, detail, backups (T012-T016) ([dee0ea6](https://github.com/nightwatch-astro/astro-up/commit/dee0ea6aa799ee5aaab7728b36f73bcfd555f1ff))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **023:** lifecycle testing — E2E discover/install/detect/uninstall CI pipeline ([#727](https://github.com/nightwatch-astro/astro-up/issues/727)) ([b2e6709](https://github.com/nightwatch-astro/astro-up/commit/b2e670953dae6bb780c571643399f49beca1f1b6))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* docs site, CLI fixes, integration tests ([#656](https://github.com/nightwatch-astro/astro-up/issues/656)) ([28855fc](https://github.com/nightwatch-astro/astro-up/commit/28855fc22f44422129c8685b8b275cec89eae862))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **015:** exit code 2 for cancellation, --quiet suppresses tracing, log dir path ([25ceb51](https://github.com/nightwatch-astro/astro-up/commit/25ceb5120584ced06cabc86ecf103f6dd717e465))
* **023:** resolve clippy warnings from lifecycle testing merge ([#729](https://github.com/nightwatch-astro/astro-up/issues/729)) ([a956f4a](https://github.com/nightwatch-astro/astro-up/commit/a956f4a9585f120df91c936431054114c1c8f540))
* avoid catalog lock contention in global_json_flag_accepted test ([#801](https://github.com/nightwatch-astro/astro-up/issues/801)) ([21a146d](https://github.com/nightwatch-astro/astro-up/commit/21a146d4f26635341060c15b2385a6b4ffff9579))
* bump to 0.2.1 to unstick release-plz after failed 0.2.0 publish ([b5b4dc9](https://github.com/nightwatch-astro/astro-up/commit/b5b4dc971804756d4eae846206f520d1254cb912))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* **cli:** rewrite self-update to use self_update crate ([#769](https://github.com/nightwatch-astro/astro-up/issues/769)) ([7250835](https://github.com/nightwatch-astro/astro-up/commit/72508357c5ab01a1289b30181f1d48978c2a9edc))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* ignore json_update_valid test (hangs on CI) ([#799](https://github.com/nightwatch-astro/astro-up/issues/799)) ([c02aa8e](https://github.com/nightwatch-astro/astro-up/commit/c02aa8e10ccdbcc8308f57ffbac17df412b3ec37))
* log astro-up version at startup for diagnostics ([#765](https://github.com/nightwatch-astro/astro-up/issues/765)) ([bafc4e6](https://github.com/nightwatch-astro/astro-up/commit/bafc4e6dfdf1a541736808c8c24ad425531ed829))
* release workflow — CLI binary builds, handover file, pnpm ([#732](https://github.com/nightwatch-astro/astro-up/issues/732)) ([147c225](https://github.com/nightwatch-astro/astro-up/commit/147c225ef8b5aa474f71dc38eba4dd06c483989a))
* revert manual version bump, let release-plz manage versions ([c0dab2d](https://github.com/nightwatch-astro/astro-up/commit/c0dab2dbaa4d619500cef7a4a560ffcd0ce319b5))
* rustfmt tracing::info macro call ([#767](https://github.com/nightwatch-astro/astro-up/issues/767)) ([75c14ef](https://github.com/nightwatch-astro/astro-up/commit/75c14efc12620cc8068b89ce229b3b50123ef32f))
* skip GitHub release for astro-up-core, fix download link ([#746](https://github.com/nightwatch-astro/astro-up/issues/746)) ([ed75b43](https://github.com/nightwatch-astro/astro-up/commit/ed75b434a430f63899b694e26b209499cb9c20b0))


### Documentation

* add README.md to core and cli crates for crates.io ([#741](https://github.com/nightwatch-astro/astro-up/issues/741)) ([332bd6a](https://github.com/nightwatch-astro/astro-up/commit/332bd6a258bf79eb331d20093f8b28041418d2dc))


### Testing

* **015:** deferred integration tests + logging fix (T036-T037) ([#462](https://github.com/nightwatch-astro/astro-up/issues/462)) ([63f5cd5](https://github.com/nightwatch-astro/astro-up/commit/63f5cd55ddc6020056224bdbad822202d16f2939))

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
