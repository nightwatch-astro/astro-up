# Tasks: Configuration System

**Input**: Design documents from `specs/004-configuration-system/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US3)

## Phase 1: Setup

**Purpose**: Add dependencies and create module structure

- [x] T001 Add workspace dependency to `Cargo.toml`: `garde = { version = "0.22", features = ["derive", "url"] }`, `humantime = "2"`
- [x] T002 Add dependencies to `crates/astro-up-core/Cargo.toml`: `garde.workspace = true`, `humantime.workspace = true`. Ensure `rusqlite` is already present.
- [x] T003 Create module structure: `crates/astro-up-core/src/config/mod.rs`, `config/model.rs`, `config/defaults.rs`, `config/store.rs`, `config/api.rs` as files with module declarations. Add `pub mod config` to `crates/astro-up-core/src/lib.rs`.
- [x] T004 Create test module structure: `crates/astro-up-core/tests/config/mod.rs` with submodule declarations.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Config structs, enums, defaults, and SQLite store needed by all user stories

- [x] T005 Implement `LogLevel` enum in `crates/astro-up-core/src/config/model.rs`: `Error`, `Warn`, `Info`, `Debug`, `Trace` with `#[serde(rename_all = "lowercase")]`, derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Display`, `EnumString`. Default: `Info`. Add method `to_tracing_level(&self) -> tracing::Level`.
- [x] T006 [P] Implement section structs in `crates/astro-up-core/src/config/model.rs`: `CatalogConfig`, `PathsConfig`, `NetworkConfig`, `UpdateConfig`, `LogConfig`, `TelemetryConfig` with all fields from data-model.md. All derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Validate`. Use `#[garde(dive)]`, `#[garde(url)]`, `#[garde(allow_unvalidated)]`, and custom validator for positive Duration.
- [x] T007 Implement `AppConfig` struct in `crates/astro-up-core/src/config/model.rs`: contains all section structs with `#[garde(dive)]`. Derive `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`, `Validate`. Add `KNOWN_KEYS: &[&str]` constant listing all valid dot-path keys (e.g., `"network.timeout"`, `"catalog.url"`).
- [x] T008 Implement `Default` impls in `crates/astro-up-core/src/config/defaults.rs` for all config structs. Duration defaults use concrete values (`Duration::from_secs(86400)` for 24h, `Duration::from_secs(30)` for timeout). Path defaults use `PathBuf::default()` (empty) — the caller injects resolved platform paths via `AppConfig::with_paths(paths: PathsConfig)` before merging SQLite overrides. `user_agent` defaults to `format!("astro-up/{}", env!("CARGO_PKG_VERSION"))`. Add `AppConfig::with_paths(paths: PathsConfig) -> Self` that returns defaults with paths populated.
- [x] T009 Implement `ConfigStore` in `crates/astro-up-core/src/config/store.rs`: struct wrapping `rusqlite::Connection`. `ConfigStore::new(conn) -> Result<Self>` creates the `config_settings` table if not exists. Methods: `get(key) -> Result<Option<String>>`, `set(key, value) -> Result<()>` (INSERT OR REPLACE), `list() -> Result<Vec<(String, String)>>`, `reset(key) -> Result<()>` (DELETE), `reset_all() -> Result<()>`.
- [x] T010 Verify all structs and ConfigStore compile with `cargo test -p astro-up-core`

**Checkpoint**: All config structs compile with garde derives, ConfigStore creates table and handles CRUD.

---

## Phase 3: User Story 1 — Default Configuration (Priority: P1)

**Goal**: Application starts and operates with zero stored config

**Independent Test**: Call `load_config(db_path, &[])` with an empty database. Assert all fields match defaults.

- [ ] T011 [US1] Implement `load_config(db_path: &Path, default_paths: PathsConfig, cli_overrides: &[(&str, &str)]) -> Result<AppConfig>` in `crates/astro-up-core/src/config/mod.rs`: open SQLite connection, create `ConfigStore`, build `AppConfig::with_paths(default_paths)`, merge stored settings from `ConfigStore::list()` via `merge_stored`, merge CLI overrides via `merge_stored`, run `config.validate()`. If SQLite open fails with corruption, log warning, rename to `*.corrupt`, create fresh DB, continue with defaults. Re-export `AppConfig`, `ConfigStore`, `load_config` from mod.rs.
- [ ] T012 [P] [US1] Implement merge logic: helper function `merge_stored(config: &mut AppConfig, stored: &[(String, String)]) -> Result<()>` that maps dot-path keys to struct fields and parses values. Type-specific parsing: `humantime::parse_duration` for Duration fields, `str::parse::<bool>` for booleans, `FromStr` (strum `EnumString`) for LogLevel, `PathBuf::from` for paths, string as-is for String fields. Error on unknown keys.
- [ ] T013 [US1] Add integration test in `crates/astro-up-core/tests/config/defaults_test.rs`: create tempfile SQLite DB, call `load_config` with empty DB and no CLI overrides, assert all fields match spec defaults table. Snapshot full config with insta.
- [ ] T014 [US1] Add integration test in `crates/astro-up-core/tests/config/validation_test.rs`: construct an `AppConfig` with invalid values (zero timeout, empty user_agent, invalid proxy URL), call `validate()`, assert errors contain field paths.

**Checkpoint**: `load_config` returns valid defaults with empty database. SC-001 passes.

---

## Phase 4: User Story 2 — Persistent Config Changes (Priority: P2)

**Goal**: `config set` persists to SQLite, `config get/list/reset` work correctly

**Independent Test**: Call `config_set(store, "network.timeout", "60s")`, then `config_get`, assert `"60s"`.

- [ ] T015 [US2] Implement `config_set(store: &ConfigStore, config: &AppConfig, key: &str, value: &str) -> Result<()>` in `crates/astro-up-core/src/config/api.rs`: validate key exists in `KNOWN_KEYS`, parse value to target type, build temporary AppConfig with the change, run garde validation, persist via `store.set()`.
- [ ] T016 [P] [US2] Implement `config_get(store: &ConfigStore, config: &AppConfig, key: &str) -> Result<String>` in `crates/astro-up-core/src/config/api.rs`: return stored value if present, otherwise format the default value from AppConfig. Validate key exists in `KNOWN_KEYS`.
- [ ] T017 [P] [US2] Implement `config_list(config: &AppConfig, stored: &[(String, String)]) -> Vec<(String, String, bool)>` in `crates/astro-up-core/src/config/api.rs`: return all keys with effective values and a flag indicating whether the value is the default or stored override.
- [ ] T018 [P] [US2] Implement `config_reset(store: &ConfigStore, key: &str) -> Result<()>` in `crates/astro-up-core/src/config/api.rs`: validate key, call `store.reset(key)`. No-op if key was never set.
- [ ] T019 [US2] Add integration test in `crates/astro-up-core/tests/config/api_test.rs`: set `network.timeout` to `60s`, get it back, assert `60s`. List all settings, assert timeout shows as overridden. Reset timeout, get again, assert default `30s`. (SC-007)
- [ ] T020 [P] [US2] Add integration test in `crates/astro-up-core/tests/config/store_test.rs`: test ConfigStore CRUD: set a key, get it, list all, reset it, verify deleted. Test auto-create of table on fresh connection. (FR-013, FR-015)
- [ ] T021 [US2] Add integration test for validation on set: call `config_set` with `network.timeout = "0s"` (zero duration), assert garde validation error. Call with unknown key `"nonexistent.field"`, assert error listing valid keys.

**Checkpoint**: Config API works end-to-end with SQLite persistence. SC-004, SC-007 pass.

---

## Phase 5: User Story 3 — CLI Flag Overrides (Priority: P3)

**Goal**: CLI flags take highest precedence over SQLite and defaults

**Independent Test**: Store `logging.level = warn` in SQLite, call `load_config` with CLI override `("logging.level", "debug")`, assert level is Debug.

- [ ] T022 [US3] Add integration test in `crates/astro-up-core/tests/config/layering_test.rs`: set `logging.level` to `warn` in SQLite, call `load_config` with CLI override `("logging.level", "debug")`, assert Debug wins. Remove CLI override, assert SQLite `warn` wins. Reset SQLite, assert default `info` wins. (SC-006)

**Checkpoint**: 3-layer precedence verified. SC-006 passes.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Error types, edge cases, final quality checks

- [ ] T023 Add edge case tests in `crates/astro-up-core/tests/config/edge_cases_test.rs`: `config get` on valid key never set → returns default; `config reset` on key never set → no error; `config set` with wrong type (`auto_check = "42"`) → type error; `config list` with no stored settings → all defaults; corrupt SQLite file (write garbage bytes to .db) → `load_config` recovers with defaults, renames corrupt file to `*.corrupt`, creates fresh DB.
- [ ] T024 [P] Add `config::Error` variants to `crates/astro-up-core/src/error.rs`: `ConfigValidation(garde::Report)`, `ConfigUnknownKey(String)`, `ConfigParse { key: String, expected: String, got: String }`, `ConfigStore(rusqlite::Error)`. Wire into API functions.
- [ ] T025 Run `cargo clippy -p astro-up-core -- -D warnings` and fix any warnings
- [ ] T026 Run `cargo test -p astro-up-core` and verify all tests pass
- [ ] T027 Run `just check` to verify workspace-wide quality gates still pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup (T001-T004)
- **US1 (Phase 3)**: Depends on Foundational — uses AppConfig, ConfigStore, load_config
- **US2 (Phase 4)**: Depends on US1 — builds config API on top of load_config
- **US3 (Phase 5)**: Depends on US1 — tests CLI override layer in load_config
- **Polish (Phase 6)**: Depends on all user stories

### Parallel Opportunities

Within Phase 2:
```
T005 (LogLevel) | T006 (section structs) — after T003 module structure
T009 (ConfigStore) — independent of T005-T007
```

Within Phase 4:
```
T016 (config_get) | T017 (config_list) | T018 (config_reset) — independent API functions
T020 (store_test) — independent of API tests
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Setup (T001-T004)
2. Foundational structs + ConfigStore (T005-T010)
3. US1: load_config with defaults (T011-T014)
4. **STOP and VALIDATE**: `load_config(db_path, &[])` returns valid defaults

### Incremental Delivery

1. Setup + Foundational → structs compile, ConfigStore handles CRUD
2. US1 → defaults work, validation works (MVP)
3. US2 → config get/set/list/reset with SQLite persistence
4. US3 → CLI flag overrides take highest precedence
5. Polish → error types, clippy, workspace check
