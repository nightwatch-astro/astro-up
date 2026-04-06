# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5](https://github.com/nightwatch-astro/astro-up/compare/astro-up-gui-v0.1.4...astro-up-gui-v0.1.5) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **016:** setup + foundational — plugins, commands, state, events (T001-T009) ([5511a83](https://github.com/nightwatch-astro/astro-up/commit/5511a8373c08f1bff2c91b3da6a8110eeae1d6f1))
* **016:** Tauri v2 app shell — commands, tray, theme, events ([#506](https://github.com/nightwatch-astro/astro-up/issues/506)) ([4dd7663](https://github.com/nightwatch-astro/astro-up/commit/4dd7663d5b90d2cd77712ec5539f6ac67174b7ed))
* **016:** US1-US4 — tray, commands, theme, error handling (T010-T027) ([7ef8bd4](https://github.com/nightwatch-astro/astro-up/commit/7ef8bd4c08a19b3adf98734980982967735811ba))
* **016:** US5 + polish — self-update, tracing, quality gates (T028-T036) ([2f86e28](https://github.com/nightwatch-astro/astro-up/commit/2f86e2815fe7e5f060fbd7fb8cb96b6c45851e03))
* **016:** wire catalog, config, backup commands to core crate ([06ea836](https://github.com/nightwatch-astro/astro-up/commit/06ea836fe89f4a9da78f3509953f45efb1c4997d))
* **022:** Vue 3 frontend — views, components, Tauri wiring ([#654](https://github.com/nightwatch-astro/astro-up/issues/654)) ([13eade7](https://github.com/nightwatch-astro/astro-up/commit/13eade713e6b64e86e505b8d4e6beced00dd3971))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))
* show release notes in update dialog and GitHub releases ([#809](https://github.com/nightwatch-astro/astro-up/issues/809)) ([6fb4c96](https://github.com/nightwatch-astro/astro-up/commit/6fb4c96cef83d61e050c8ba7cff0f77b913d2942))


### Bug Fixes

* **001:** fix tauri schema URL, gitignore gen/ directory ([801c0e9](https://github.com/nightwatch-astro/astro-up/commit/801c0e91cbf5b4f4e15f2a299593ff1e1d06c377))
* **001:** track icon.ico/icon.png, add Tauri system deps to CI ([4b92383](https://github.com/nightwatch-astro/astro-up/commit/4b92383a33c70a23ccf7b0077ee7410321defb5f))
* **016:** resolve phantom completions and partial implementations ([d59f1b1](https://github.com/nightwatch-astro/astro-up/commit/d59f1b1faef8b14f832524ab0166e7c88bb1f367))
* **016:** wire tray update check and badge count from sync findings ([00754bb](https://github.com/nightwatch-astro/astro-up/commit/00754bb918c263b72bc14aa93c7fca075d9e30e5))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* PrimeVue deprecations + window size enforcement ([#655](https://github.com/nightwatch-astro/astro-up/issues/655)) ([6d4a72e](https://github.com/nightwatch-astro/astro-up/commit/6d4a72ec978ca2c6eb7e17ec5f39efc0f518f62f))
* **release:** fix tauri-action pnpm path resolution ([#811](https://github.com/nightwatch-astro/astro-up/issues/811)) ([f684a07](https://github.com/nightwatch-astro/astro-up/commit/f684a07c76c632f8e224329b40d553f9faba912c))


### Documentation

* **017:** add frontend mockup, design document, and min window size ([59e8f58](https://github.com/nightwatch-astro/astro-up/commit/59e8f583305206801d06a52f52a49244c495eceb))

## [0.1.4](https://github.com/nightwatch-astro/astro-up/compare/astro-up-gui-v0.1.3...astro-up-gui-v0.1.4) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **016:** setup + foundational — plugins, commands, state, events (T001-T009) ([5511a83](https://github.com/nightwatch-astro/astro-up/commit/5511a8373c08f1bff2c91b3da6a8110eeae1d6f1))
* **016:** Tauri v2 app shell — commands, tray, theme, events ([#506](https://github.com/nightwatch-astro/astro-up/issues/506)) ([4dd7663](https://github.com/nightwatch-astro/astro-up/commit/4dd7663d5b90d2cd77712ec5539f6ac67174b7ed))
* **016:** US1-US4 — tray, commands, theme, error handling (T010-T027) ([7ef8bd4](https://github.com/nightwatch-astro/astro-up/commit/7ef8bd4c08a19b3adf98734980982967735811ba))
* **016:** US5 + polish — self-update, tracing, quality gates (T028-T036) ([2f86e28](https://github.com/nightwatch-astro/astro-up/commit/2f86e2815fe7e5f060fbd7fb8cb96b6c45851e03))
* **016:** wire catalog, config, backup commands to core crate ([06ea836](https://github.com/nightwatch-astro/astro-up/commit/06ea836fe89f4a9da78f3509953f45efb1c4997d))
* **022:** Vue 3 frontend — views, components, Tauri wiring ([#654](https://github.com/nightwatch-astro/astro-up/issues/654)) ([13eade7](https://github.com/nightwatch-astro/astro-up/commit/13eade713e6b64e86e505b8d4e6beced00dd3971))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **001:** fix tauri schema URL, gitignore gen/ directory ([801c0e9](https://github.com/nightwatch-astro/astro-up/commit/801c0e91cbf5b4f4e15f2a299593ff1e1d06c377))
* **001:** track icon.ico/icon.png, add Tauri system deps to CI ([4b92383](https://github.com/nightwatch-astro/astro-up/commit/4b92383a33c70a23ccf7b0077ee7410321defb5f))
* **016:** resolve phantom completions and partial implementations ([d59f1b1](https://github.com/nightwatch-astro/astro-up/commit/d59f1b1faef8b14f832524ab0166e7c88bb1f367))
* **016:** wire tray update check and badge count from sync findings ([00754bb](https://github.com/nightwatch-astro/astro-up/commit/00754bb918c263b72bc14aa93c7fca075d9e30e5))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* PrimeVue deprecations + window size enforcement ([#655](https://github.com/nightwatch-astro/astro-up/issues/655)) ([6d4a72e](https://github.com/nightwatch-astro/astro-up/commit/6d4a72ec978ca2c6eb7e17ec5f39efc0f518f62f))


### Documentation

* **017:** add frontend mockup, design document, and min window size ([59e8f58](https://github.com/nightwatch-astro/astro-up/commit/59e8f583305206801d06a52f52a49244c495eceb))

## [0.1.3](https://github.com/nightwatch-astro/astro-up/compare/astro-up-gui-v0.1.2...astro-up-gui-v0.1.3) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **016:** setup + foundational — plugins, commands, state, events (T001-T009) ([5511a83](https://github.com/nightwatch-astro/astro-up/commit/5511a8373c08f1bff2c91b3da6a8110eeae1d6f1))
* **016:** Tauri v2 app shell — commands, tray, theme, events ([#506](https://github.com/nightwatch-astro/astro-up/issues/506)) ([4dd7663](https://github.com/nightwatch-astro/astro-up/commit/4dd7663d5b90d2cd77712ec5539f6ac67174b7ed))
* **016:** US1-US4 — tray, commands, theme, error handling (T010-T027) ([7ef8bd4](https://github.com/nightwatch-astro/astro-up/commit/7ef8bd4c08a19b3adf98734980982967735811ba))
* **016:** US5 + polish — self-update, tracing, quality gates (T028-T036) ([2f86e28](https://github.com/nightwatch-astro/astro-up/commit/2f86e2815fe7e5f060fbd7fb8cb96b6c45851e03))
* **016:** wire catalog, config, backup commands to core crate ([06ea836](https://github.com/nightwatch-astro/astro-up/commit/06ea836fe89f4a9da78f3509953f45efb1c4997d))
* **022:** Vue 3 frontend — views, components, Tauri wiring ([#654](https://github.com/nightwatch-astro/astro-up/issues/654)) ([13eade7](https://github.com/nightwatch-astro/astro-up/commit/13eade713e6b64e86e505b8d4e6beced00dd3971))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **001:** fix tauri schema URL, gitignore gen/ directory ([801c0e9](https://github.com/nightwatch-astro/astro-up/commit/801c0e91cbf5b4f4e15f2a299593ff1e1d06c377))
* **001:** track icon.ico/icon.png, add Tauri system deps to CI ([4b92383](https://github.com/nightwatch-astro/astro-up/commit/4b92383a33c70a23ccf7b0077ee7410321defb5f))
* **016:** resolve phantom completions and partial implementations ([d59f1b1](https://github.com/nightwatch-astro/astro-up/commit/d59f1b1faef8b14f832524ab0166e7c88bb1f367))
* **016:** wire tray update check and badge count from sync findings ([00754bb](https://github.com/nightwatch-astro/astro-up/commit/00754bb918c263b72bc14aa93c7fca075d9e30e5))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* PrimeVue deprecations + window size enforcement ([#655](https://github.com/nightwatch-astro/astro-up/issues/655)) ([6d4a72e](https://github.com/nightwatch-astro/astro-up/commit/6d4a72ec978ca2c6eb7e17ec5f39efc0f518f62f))


### Documentation

* **017:** add frontend mockup, design document, and min window size ([59e8f58](https://github.com/nightwatch-astro/astro-up/commit/59e8f583305206801d06a52f52a49244c495eceb))

## [0.1.2](https://github.com/nightwatch-astro/astro-up/compare/astro-up-gui-v0.1.1...astro-up-gui-v0.1.2) (2026-04-06)


### Features

* **001:** T004-T009, T019 — create Rust crates (core, cli, gui) ([35170bc](https://github.com/nightwatch-astro/astro-up/commit/35170bccd280ff0c815a0eeb086051d66ca58824)), closes [#2](https://github.com/nightwatch-astro/astro-up/issues/2)
* **016:** setup + foundational — plugins, commands, state, events (T001-T009) ([5511a83](https://github.com/nightwatch-astro/astro-up/commit/5511a8373c08f1bff2c91b3da6a8110eeae1d6f1))
* **016:** Tauri v2 app shell — commands, tray, theme, events ([#506](https://github.com/nightwatch-astro/astro-up/issues/506)) ([4dd7663](https://github.com/nightwatch-astro/astro-up/commit/4dd7663d5b90d2cd77712ec5539f6ac67174b7ed))
* **016:** US1-US4 — tray, commands, theme, error handling (T010-T027) ([7ef8bd4](https://github.com/nightwatch-astro/astro-up/commit/7ef8bd4c08a19b3adf98734980982967735811ba))
* **016:** US5 + polish — self-update, tracing, quality gates (T028-T036) ([2f86e28](https://github.com/nightwatch-astro/astro-up/commit/2f86e2815fe7e5f060fbd7fb8cb96b6c45851e03))
* **016:** wire catalog, config, backup commands to core crate ([06ea836](https://github.com/nightwatch-astro/astro-up/commit/06ea836fe89f4a9da78f3509953f45efb1c4997d))
* **022:** Vue 3 frontend — views, components, Tauri wiring ([#654](https://github.com/nightwatch-astro/astro-up/issues/654)) ([13eade7](https://github.com/nightwatch-astro/astro-up/commit/13eade713e6b64e86e505b8d4e6beced00dd3971))
* **023:** complete lifecycle testing — tests, ledger, catalog schema, docs ([#738](https://github.com/nightwatch-astro/astro-up/issues/738)) ([8d57ead](https://github.com/nightwatch-astro/astro-up/commit/8d57ead10fcf9222c961542c50fc7dcc0cc36257))
* **024:** wire all CLI commands to astro-up-core engine ([#716](https://github.com/nightwatch-astro/astro-up/issues/716)) ([cda4c05](https://github.com/nightwatch-astro/astro-up/commit/cda4c05acedbc041b15a23f7e30cf29430171efd))
* merge spec 001 — repository scaffolding ([208b7fb](https://github.com/nightwatch-astro/astro-up/commit/208b7fbd9eea8cb31fed2ac551d8a573879f096f))
* merge spec 001 — repository scaffolding (complete) ([9456350](https://github.com/nightwatch-astro/astro-up/commit/9456350b5eb0df9f9f47da16f421700979d2a745))
* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))


### Bug Fixes

* **001:** fix tauri schema URL, gitignore gen/ directory ([801c0e9](https://github.com/nightwatch-astro/astro-up/commit/801c0e91cbf5b4f4e15f2a299593ff1e1d06c377))
* **001:** track icon.ico/icon.png, add Tauri system deps to CI ([4b92383](https://github.com/nightwatch-astro/astro-up/commit/4b92383a33c70a23ccf7b0077ee7410321defb5f))
* **016:** resolve phantom completions and partial implementations ([d59f1b1](https://github.com/nightwatch-astro/astro-up/commit/d59f1b1faef8b14f832524ab0166e7c88bb1f367))
* **016:** wire tray update check and badge count from sync findings ([00754bb](https://github.com/nightwatch-astro/astro-up/commit/00754bb918c263b72bc14aa93c7fca075d9e30e5))
* **ci:** inline release workflow, add Tauri build, skip crates.io ([#164](https://github.com/nightwatch-astro/astro-up/issues/164)) ([4504ebe](https://github.com/nightwatch-astro/astro-up/commit/4504ebeb0e66293c1419e22a5d405faac271dd53))
* delete etag sidecars during purge, use config download_dir ([#753](https://github.com/nightwatch-astro/astro-up/issues/753)) ([a8e3235](https://github.com/nightwatch-astro/astro-up/commit/a8e323575cf93cc6d298bc45dec49f6d2a756ec4))
* PrimeVue deprecations + window size enforcement ([#655](https://github.com/nightwatch-astro/astro-up/issues/655)) ([6d4a72e](https://github.com/nightwatch-astro/astro-up/commit/6d4a72ec978ca2c6eb7e17ec5f39efc0f518f62f))


### Documentation

* **017:** add frontend mockup, design document, and min window size ([59e8f58](https://github.com/nightwatch-astro/astro-up/commit/59e8f583305206801d06a52f52a49244c495eceb))

## [0.1.1](https://github.com/nightwatch-astro/astro-up/compare/astro-up-gui-v0.1.0...astro-up-gui-v0.1.1) (2026-04-06)


### Features

* **release:** bundle CLI as sidecar in Tauri NSIS installer ([#802](https://github.com/nightwatch-astro/astro-up/issues/802)) ([5891aa8](https://github.com/nightwatch-astro/astro-up/commit/5891aa88baf86bb6d3240f869ee61b7b6b37eece))

## [Unreleased]

## [0.1.0](https://github.com/nightwatch-astro/astro-up/releases/tag/astro-up-gui-v0.1.0) - 2026-03-30

### Bug Fixes

- *(001)* track icon.ico/icon.png, add Tauri system deps to CI
- *(001)* fix tauri schema URL, gitignore gen/ directory

### Features

- *(001)* T004-T009, T019 — create Rust crates (core, cli, gui)

### Miscellaneous

- untrack generated files, update .gitignore
