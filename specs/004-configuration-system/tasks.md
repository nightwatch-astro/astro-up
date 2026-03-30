# Tasks: Configuration System

**Input**: Design documents from `specs/004-configuration-system/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US4)

## Phase 1: Setup

**Purpose**: Add dependencies and create module structure

- [ ] T001 Add workspace dependencies to `Cargo.toml`: `config = { version = "0.15", default-features = false, features = ["toml", "convert-case"] }`, `garde = { version = "0.22", features = ["derive", "url"] }`, `humantime-serde = "1.1"`, `directories = "6.0"`
- [ ] T002 Add dependencies to `crates/astro-up-core/Cargo.toml`: `config.workspace = true`, `garde.workspace = true`, `humantime-serde.workspace = true`, `directories.workspace = true`. Move `toml` from dev-dependencies to dependencies.
- [ ] T003 Create module structure: `crates/astro-up-core/src/config/mod.rs`, `config/model.rs`, `config/defaults.rs`, `config/tokens.rs`, `config/unknown_keys.rs`, `config/init.rs` as files with module declarations. Add `pub mod config` to `crates/astro-up-core/src/lib.rs`.
- [ ] T004 Create test module structure: `crates/astro-up-core/tests/config/mod.rs` with submodule declarations. Add `mod config;` to test runner if needed.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Config structs, enums, defaults, and token resolver needed by all user stories

- [ ] T005 Implement `LogLevel` enum in `crates/astro-up-core/src/config/model.rs`: `Error`, `Warn`, `Info`, `Debug`, `Trace` with `#[serde(rename_all = "lowercase")]`, derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Display`, `EnumString`. Default: `Info`.
- [ ] T006 [P] Implement section structs in `crates/astro-up-core/src/config/model.rs`: `CatalogConfig` (`url: String`, `cache_ttl: Duration` with humantime_serde, `offline: bool`), `PathsConfig` (`download_dir: PathBuf`, `cache_dir: PathBuf`, `data_dir: PathBuf`), `NetworkConfig` (`proxy: Option<String>`, `timeout: Duration` with humantime_serde, `user_agent: String`), `UpdateConfig` (`auto_check: bool`, `check_interval: Duration` with humantime_serde), `LogConfig` (`level: LogLevel`, `log_to_file: bool`, `log_file: PathBuf`), `TelemetryConfig` (`enabled: bool`). All derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Validate`. Use `#[serde(default)]` on each struct. Use `#[garde(dive)]`, `#[garde(url)]`, `#[garde(allow_unvalidated)]`, and custom validators for positive Duration.
- [ ] T007 Implement `AppConfig` struct in `crates/astro-up-core/src/config/model.rs`: contains all section structs as fields with `#[serde(default)]` and `#[garde(dive)]`. Derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Validate`.
- [ ] T008 Implement `TokenResolver` in `crates/astro-up-core/src/config/tokens.rs`: struct holding resolved `config_dir`, `cache_dir`, `data_dir`, `home_dir` as `PathBuf`. `TokenResolver::new() -> Result<Self>` using `directories::ProjectDirs::from("", "", "astro-up")` and `BaseDirs::new()` — return `CoreError::HomeDirNotFound` if either returns `None`. `expand(&self, input: &str) -> Result<PathBuf>` replaces `{config_dir}` etc., errors on unknown tokens. `expand_config(&self, config: &mut AppConfig) -> Result<()>` expands exactly 4 fields: `paths.download_dir`, `paths.cache_dir`, `paths.data_dir`, `logging.log_file` (explicit list, Rust has no runtime reflection).
- [ ] T009 Implement `Default` impls in `crates/astro-up-core/src/config/defaults.rs` for all config structs. Two patterns: (1) path fields use token syntax strings pre-expansion (e.g., `PathBuf::from("{cache_dir}/downloads")` for `download_dir`, `PathBuf::from("{data_dir}/astro-up.log")` for `log_file`), (2) scalar fields use concrete values (e.g., `Duration::from_secs(86400)` for 24h cache_ttl, `Duration::from_secs(30)` for timeout, `LogLevel::Info` for level). `user_agent` defaults to `format!("astro-up/{}", env!("CARGO_PKG_VERSION"))`.
- [ ] T010 Verify all structs compile and round-trip with `cargo test -p astro-up-core`

**Checkpoint**: All config structs compile with serde + garde derives, TokenResolver resolves platform paths.

---

## Phase 3: User Story 1 — Default Configuration (Priority: P1)

**Goal**: Application starts and operates with zero configuration — no file, no env vars

**Independent Test**: Call `load_config(None, &[])` with no config file and no env vars. Assert all fields match the defaults table.

- [ ] T011 [US1] Implement `load_config(cli_config_path: Option<&Path>, cli_overrides: &[(&str, &str)]) -> Result<AppConfig>` in `crates/astro-up-core/src/config/mod.rs`: build `Config::builder()`, set defaults from `AppConfig::default()` via `Config::try_from(&defaults)?`, add file source (if file exists), add `Environment::with_prefix("ASTROUP").separator("__").try_parsing(true)`, apply CLI overrides via `set_override`, `.build()?.try_deserialize::<AppConfig>()`, then `TokenResolver::expand_config()`, then `config.validate()`. Re-export `AppConfig`, `load_config` from mod.rs.
- [ ] T012 [US1] Add integration test in `crates/astro-up-core/tests/config/defaults_test.rs`: call `load_config(None, &[])` in a tempdir with no config file, assert all fields match the spec defaults table. Snapshot the full config with insta.
- [ ] T013 [US1] Add integration test in `crates/astro-up-core/tests/config/tokens_test.rs`: call `load_config(None, &[])`, assert all PathBuf fields are absolute paths (no `{` tokens remaining). Assert `config_dir` contains "astro-up".
- [ ] T014 [US1] Add integration test in `crates/astro-up-core/tests/config/validation_test.rs`: construct an `AppConfig` with invalid values (zero timeout, empty user_agent, invalid proxy URL), call `validate()`, assert errors contain field paths matching `"config.{section}.{field}"` format.

**Checkpoint**: `load_config(None, &[])` returns valid defaults with resolved paths. SC-001 passes.

---

## Phase 4: User Story 2 — TOML Config File (Priority: P2)

**Goal**: Power user creates config.toml, custom values override defaults

**Independent Test**: Create a config.toml with `[network]\ntimeout = "60s"`, call `load_config`, assert timeout is 60s.

- [ ] T015 [US2] Implement `check_unknown_keys(file_path: &Path) -> Vec<String>` in `crates/astro-up-core/src/config/unknown_keys.rs`: read file, parse with `toml::from_str::<toml::Value>()`, collect all keys recursively, diff against known `AppConfig` field names (hardcoded set), return list of unknown keys. Log each as `tracing::warn!`.
- [ ] T016 [US2] Wire `check_unknown_keys` into `load_config`: after `try_deserialize`, if a config file was loaded, call `check_unknown_keys` with the file path.
- [ ] T017 [US2] Implement `generate_config_template() -> String` in `crates/astro-up-core/src/config/init.rs`: generate a fully-documented TOML string with all settings commented out at their default values, with explanatory comments per section. Use `toml::to_string_pretty(&AppConfig::default())` as base, then prefix each line with `# `.
- [ ] T018 [US2] Implement `show_effective_config(config: &AppConfig) -> Result<String>` in `crates/astro-up-core/src/config/init.rs`: serialize the post-expansion config to pretty TOML via `toml::to_string_pretty`.
- [ ] T019 [US2] Add integration test in `crates/astro-up-core/tests/config/roundtrip_test.rs`: load defaults → serialize to TOML string → write to tempfile → load from file → assert equality (SC-005). Use pre-expansion config for comparison.
- [ ] T020 [P] [US2] Add integration test in `crates/astro-up-core/tests/config/unknown_keys_test.rs`: write a TOML with known + unknown keys (`typo_field = "oops"`), call `check_unknown_keys`, assert the unknown key is returned. Also test that valid keys are NOT flagged.
- [ ] T021 [P] [US2] Add snapshot test in `crates/astro-up-core/tests/config/init_test.rs`: call `generate_config_template()`, snapshot the output with insta. Assert it contains all section headers and field names.
- [ ] T022 [US2] Add integration test for TOML override: write a config.toml with `[network]\ntimeout = "60s"`, load config from that file, assert `network.timeout == Duration::from_secs(60)` while all other fields are defaults.
- [ ] T023 [US2] Add integration test for validation errors: write a config.toml with `[network]\ntimeout = "0s"`, load config, assert garde validation error mentions `timeout`.

**Checkpoint**: TOML file overrides defaults, unknown keys warned, config init/show work. SC-004, SC-005 pass.

---

## Phase 5: User Story 3 — Environment Variable Overrides (Priority: P3)

**Goal**: Env vars with `ASTROUP_` prefix override file and defaults

**Independent Test**: Set `ASTROUP_CATALOG__URL=https://test.example.com`, load config, assert catalog URL matches.

- [ ] T024 [US3] Add `load_config_with_env` variant in `crates/astro-up-core/src/config/mod.rs` that accepts `Option<HashMap<String, String>>` for mock env source. When `Some`, use `Environment::source(Some(map))` instead of real env vars. Wire into `load_config` (real env) and expose for testing.
- [ ] T025 [US3] Add integration test in `crates/astro-up-core/tests/config/env_test.rs`: use `load_config_with_env` with mock env `{"ASTROUP_CATALOG__URL": "https://test.example.com"}`, assert `catalog.url == "https://test.example.com"`. Test that TOML file values are overridden by env vars.
- [ ] T026 [P] [US3] Add integration test for nested env vars: set `ASTROUP_UPDATES__CHECK_INTERVAL=6h`, assert `updates.check_interval == Duration::from_secs(21600)`. Verify single underscore in `CHECK_INTERVAL` is not split.
- [ ] T027 [P] [US3] Add integration test for unknown env vars: set `ASTROUP_NONEXISTENT__FIELD=whatever`, load config, assert no error (silently ignored per FR-002).
- [ ] T028 [US3] Add integration test for `try_parsing`: set `ASTROUP_UPDATES__AUTO_CHECK=true` (string), load config, assert `updates.auto_check == true` (parsed to bool).

**Checkpoint**: Env vars override TOML and defaults. SC-002 passes.

---

## Phase 6: User Story 4 — CLI Argument Overrides (Priority: P4)

**Goal**: CLI args take highest precedence via `set_override`

**Independent Test**: Call `load_config` with cli_overrides `[("logging.level", "debug")]`, assert level is Debug.

- [ ] T029 [US4] Add integration test in `crates/astro-up-core/tests/config/layering_test.rs`: set all four layers to different values for the same field (`logging.level`): default=info, TOML=warn, env=error, CLI=debug. Assert CLI wins. Remove CLI, assert env wins. Remove env, assert TOML wins. Remove TOML, assert default wins. (SC-006)
- [ ] T030 [US4] Add integration test for `--config` override: create two TOML files with different values, load with `cli_config_path` pointing to the second file, assert second file's values are used.
- [ ] T031 [US4] Add integration test for `--config` + env var interaction: load with custom config file AND env var override on the same field, assert env var wins (layering: CLI args > env > file > defaults, but `--config` only changes which file, not its precedence).

**Checkpoint**: CLI overrides work at highest precedence. SC-006 passes.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final quality checks, edge cases, workspace validation

- [ ] T032 Add edge case tests in `crates/astro-up-core/tests/config/edge_cases_test.rs`: empty config file (0 bytes) → defaults apply; TOML syntax error → `ConfigError::FileParse` with line/column; path token in TOML value (`download_dir = "{home_dir}/downloads"`) → expanded to absolute path; `generate_config_template()` output can be written to tempfile and loaded back without errors; `show_effective_config` on default config (no file) returns valid TOML with resolved absolute paths.
- [ ] T033 [P] Add `config::Error` variants to `crates/astro-up-core/src/error.rs`: `ConfigLoad(config::ConfigError)`, `ConfigValidation(garde::Report)`, `TokenExpansion(String)`, `HomeDirNotFound`. Wire into `load_config` error handling.
- [ ] T034 Run `cargo clippy -p astro-up-core -- -D warnings` and fix any warnings
- [ ] T035 Run `cargo test -p astro-up-core` and verify all tests pass
- [ ] T036 Run `just check` to verify workspace-wide quality gates still pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup (T001-T004)
- **US1 (Phase 3)**: Depends on Foundational — first to use `load_config`
- **US2 (Phase 4)**: Depends on US1 — adds file source + unknown keys + init/show
- **US3 (Phase 5)**: Depends on US1 — adds env source (can run parallel with US2 in theory, but the builder is additive)
- **US4 (Phase 6)**: Depends on US1 — adds CLI overrides (can run parallel with US2/US3)
- **Polish (Phase 7)**: Depends on all user stories

### Story Dependencies

User stories 2-4 are additive layers on the same `load_config` function. While conceptually independent, they share the builder pipeline:
- **US1**: defaults + token expansion + validation (the core)
- **US2**: + TOML file source + unknown key detection + init/show
- **US3**: + Environment source
- **US4**: + CLI overrides via set_override

Recommended order: US1 → US2 → US3 → US4 (each builds on the previous)

### Parallel Opportunities

Within Phase 2:
```
T005 (LogLevel) | T006 (section structs) — after T003 module structure
T008 (TokenResolver) — independent of T005-T007
```

Within Phase 4:
```
T020 (unknown keys test) | T021 (init test) — independent test files
```

Within Phase 5:
```
T026 (nested env test) | T027 (unknown env test) — independent test files
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Setup (T001-T004)
2. Foundational structs + TokenResolver (T005-T010)
3. US1: load_config with defaults (T011-T014)
4. **STOP and VALIDATE**: `load_config(None, &[])` returns valid defaults with resolved paths

### Incremental Delivery

1. Setup + Foundational → structs compile, TokenResolver resolves
2. US1 → defaults work, token expansion works, validation works (MVP)
3. US2 → TOML file overrides, unknown key warnings, config init/show
4. US3 → env var overrides with `ASTROUP_` prefix
5. US4 → CLI arg overrides take highest precedence
6. Polish → error types, clippy, workspace check
