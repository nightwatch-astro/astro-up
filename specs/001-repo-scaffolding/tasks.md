# Tasks: Repository Scaffolding

**Input**: Design documents from `specs/001-repo-scaffolding/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Root workspace configuration, toolchain pinning, and .gitignore

- [x] T001 Create workspace `Cargo.toml` at repo root with members `crates/astro-up-core`, `crates/astro-up-cli`, `crates/astro-up-gui` and shared workspace dependencies (serde, serde_json, insta)
- [x] T002 [P] Create `rust-toolchain.toml` pinning stable channel and 2024 edition
- [x] T003 [P] Create `.gitignore` covering `target/`, `node_modules/`, `frontend/dist/`, `*.swp`, `.DS_Store`, `Cargo.lock` exclusion for libraries (keep for workspace)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The three crates and frontend must exist before any user story can be validated

- [x] T004 Create `crates/astro-up-core/Cargo.toml` (lib crate, edition 2024) and `crates/astro-up-core/src/lib.rs` with a public `version()` function returning the crate version and a `#[cfg(test)]` smoke test
- [x] T005 [P] Create `crates/astro-up-cli/Cargo.toml` (lib+bin, depends on astro-up-core + clap 4 derive) and `crates/astro-up-cli/src/lib.rs` with a `run()` stub, and `crates/astro-up-cli/src/main.rs` calling `lib::run()`
- [x] T006 [P] Create `crates/astro-up-gui/Cargo.toml` (lib+bin, depends on astro-up-core + tauri 2), `crates/astro-up-gui/build.rs` calling `tauri_build::build()`, and `crates/astro-up-gui/src/lib.rs` with a `run()` function that initializes the Tauri app with an empty `invoke_handler`
- [x] T007 Create `crates/astro-up-gui/src/main.rs` calling `astro_up_gui::run()`, create `crates/astro-up-gui/tauri.conf.json` with identifier `dev.nightwatch.astro-up`, window title `Astro-Up`, default size 1024x768, `frontendDist` pointing to `../../frontend/dist`, `devUrl` to `http://localhost:5173`, `beforeDevCommand` and `beforeBuildCommand` invoking pnpm in `../../frontend`
- [x] T008 Create `crates/astro-up-gui/capabilities/default.json` granting core Tauri v2 permissions (window:default, app:default)
- [x] T009 Create default Tauri icons in `crates/astro-up-gui/icons/` (use `cargo tauri icon` or copy Tauri default set)
- [ ] T010 Initialize frontend: create `frontend/package.json` with dependencies (vue ^3, primevue ^4, @primeuix/themes ^1, @tanstack/vue-query ^5, @tauri-apps/api ^2) and devDependencies (typescript ^5, vite ^6, @vitejs/plugin-vue ^5, vitest ^3, eslint ^9, vue-tsc), then run `pnpm install` to generate lockfile
- [ ] T011 [P] Create `frontend/index.html` with Vite entry point linking to `src/main.ts`
- [ ] T012 [P] Create `frontend/vite.config.ts` with Vue plugin, Tauri host configuration (`server.strictPort`, `server.host` for mobile, clearScreen false), and Vitest config
- [ ] T013 [P] Create `frontend/tsconfig.json` and `frontend/tsconfig.node.json` for Vue + TypeScript compilation
- [ ] T014 Create `frontend/src/main.ts` initializing Vue app with PrimeVue (Aura theme, `darkModeSelector: 'system'`, `cssLayer: false`) and VueQueryPlugin, mounting to `#app`
- [ ] T015 Create `frontend/src/App.vue` with a PrimeVue Card component displaying app name and version as placeholder, confirming PrimeVue renders correctly
- [ ] T016 [P] Create `frontend/src/styles.css` with minimal global styles (body margin reset, font-family)
- [ ] T017 [P] Create `frontend/src/vite-env.d.ts` with Vite client type reference
- [ ] T018 [P] Create `frontend/eslint.config.js` with flat config for Vue + TypeScript
- [x] T019 Verify `cargo check --workspace` compiles all three crates without errors
- [ ] T020 Verify `pnpm --dir frontend build` produces `frontend/dist/` with index.html

**Checkpoint**: All crates compile, frontend builds. Ready for user story implementation.

---

## Phase 3: User Story 1 — Build and Run the Application (Priority: P1)

**Goal**: `just setup && just dev` launches Tauri window with Vue+PrimeVue placeholder

**Independent Test**: Run `just dev`, verify Tauri window opens with PrimeVue card rendered

- [ ] T021 [US1] Create `Justfile` at repo root with recipes: `setup` (runs `pnpm --dir frontend install`), `dev` (runs `cargo tauri dev`), `build` (runs `cargo tauri build`)
- [ ] T022 [US1] Run `just dev` and verify: (a) Vite dev server starts on port 5173, (b) Tauri window opens, (c) PrimeVue Card placeholder is visible, (d) Vue hot-reload works on App.vue edits, (e) Rust changes trigger recompile
- [ ] T023 [US1] Add a Vitest test in `frontend/src/App.test.ts` that verifies the App component mounts and contains the placeholder text

**Checkpoint**: US1 complete — developer can clone, setup, and run the app

---

## Phase 4: User Story 2 — Run All Quality Checks Locally (Priority: P2)

**Goal**: `just check` runs all quality gates matching CI

**Independent Test**: Run `just check` on clean scaffold — exit code 0

- [ ] T024 [US2] Add Justfile recipes: `test` (cargo test + pnpm test), `fmt` (cargo fmt --all), `lint` (cargo clippy -D warnings + pnpm lint), `check` (fmt --check + lint + test + pnpm build)
- [ ] T025 [US2] Run `just check` and verify all steps pass with exit code 0
- [ ] T026 [US2] Intentionally break Rust formatting, verify `just check` fails on `cargo fmt --check`

**Checkpoint**: US2 complete — local quality checks mirror CI

---

## Phase 5: User Story 3 — CI Validates Every Pull Request (Priority: P3)

**Goal**: GitHub Actions CI runs 3 parallel jobs on every PR

**Independent Test**: Push branch, open PR, verify CI passes

- [ ] T027 [US3] Create `.github/workflows/ci.yml` with three jobs: `check-rust` (ubuntu-latest: checkout, dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2, cargo fmt --check, cargo clippy --workspace -- -D warnings, cargo test --workspace), `check-frontend` (ubuntu-latest: checkout, setup node + pnpm with store cache, pnpm install --frozen-lockfile, pnpm lint, pnpm test, pnpm build), `check-windows` (windows-latest: checkout, dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2, cargo check --workspace, cargo test --workspace — path filter on crates/**, Cargo.toml, Cargo.lock). Add semantic PR title validation via amannn/action-semantic-pull-request@v6. Add concurrency group to cancel duplicate runs.
- [ ] T028 [P] [US3] Create `.github/dependabot.yml` with three ecosystems: cargo (weekly, crates/ directory, `chore(deps)` prefix), npm (weekly, frontend/ directory, `chore(deps)` prefix), github-actions (weekly, / directory, `chore(ci)` prefix) — matching nightwatch-astro org convention
- [ ] T029 [P] [US3] Create `release-plz.toml` with workspace config (publish, git_tag_enable, git_release_enable, dependencies_update, pr_labels=["release"]) and changelog commit_parsers for conventional commits (feat, fix, perf, refactor, docs, test, chore). Create `.github/workflows/release.yml` delegating to `nightwatch-astro/.github/.github/workflows/rust-release.yml@main` with secrets (NIGHTWATCH_APP_ID, NIGHTWATCH_APP_PRIVATE_KEY, CARGO_REGISTRY_TOKEN)
- [ ] T030 [US3] Push branch, open test PR, verify `check-rust` and `check-frontend` jobs pass (check-windows may be skipped due to path filter)

**Checkpoint**: US3 complete — CI enforces quality on every PR

---

## Phase 6: User Story 4 — Automated Dependency Updates (Priority: P4)

**Goal**: Dependabot opens PRs for Cargo and npm updates

**Independent Test**: Verify `dependabot.yml` is valid YAML and covers both ecosystems

- [ ] T031 [US4] Validate `dependabot.yml` configuration: verify YAML parses correctly, has `cargo` and `npm` package ecosystems, correct directory paths, weekly schedule

**Checkpoint**: US4 complete — Dependabot configured

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, changelog config, and branch protection

- [ ] T032 [P] Write `CLAUDE.md` with project conventions: workspace layout, Justfile recipes, test commands, branch strategy, Tauri dev workflow, frontend structure, coding standards (Rust: cargo fmt + clippy -D warnings, Vue: ESLint flat config + TypeScript strict)
- [ ] T034 Configure branch protection on `main` via `gh api`: require CI status checks (`check-rust`, `check-frontend`), require 1 PR review, enforce for admins
- [ ] T035 Run `just check` one final time to verify everything passes end-to-end

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001-T003)
- **US1 (Phase 3)**: Depends on Foundational (T004-T020) — Justfile + verify dev flow
- **US2 (Phase 4)**: Depends on US1 (T021 Justfile exists) — add check/lint/test recipes
- **US3 (Phase 5)**: Depends on US2 (CI mirrors local checks) — create workflows
- **US4 (Phase 6)**: Depends on US3 (Dependabot.yml created in T028) — validation only
- **Polish (Phase 7)**: Depends on all user stories

### Parallel Opportunities

Within Phase 2 (after T004 core crate):
```
T005 (cli crate) | T006 (gui crate)     # parallel — different directories
T011 (index.html) | T012 (vite.config) | T013 (tsconfig) | T016 (styles) | T017 (vite-env) | T018 (eslint)
```

Within Phase 5:
```
T028 (dependabot) | T029 (release-please)  # parallel — different files
```

Within Phase 7:
```
T032 (CLAUDE.md) | T034 (branch protection)  # parallel — different targets
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T020)
3. Complete Phase 3: US1 (T021-T023)
4. **STOP and VALIDATE**: `just dev` opens Tauri window with PrimeVue

### Incremental Delivery

1. Setup + Foundational → workspace compiles, frontend builds
2. US1 → `just dev` works (MVP)
3. US2 → `just check` works
4. US3 → CI enforces quality
5. US4 → Dependabot validates
6. Polish → docs, changelog, branch protection
