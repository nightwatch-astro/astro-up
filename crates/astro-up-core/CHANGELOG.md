# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
