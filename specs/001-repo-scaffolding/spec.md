# Feature Specification: Repository Scaffolding

**Feature Branch**: `001-repo-scaffolding`
**Created**: 2026-03-29
**Status**: Draft
**Project**: Rust Migration
**Project Number**: 1
**Input**: User description: "Create Cargo workspace, Tauri project, Vue frontend, CI, and initial configuration"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Build and Run the Application (Priority: P1)

A developer clones the repository and wants to build and run the application locally. They run a single command to start the Tauri development server with hot-reload on the Vue frontend. The application window opens and shows a placeholder page confirming the stack works end-to-end.

**Why this priority**: Without a working build pipeline, no other spec can be implemented. This is the foundation everything else depends on.

**Independent Test**: Clone the repo on a clean machine, run `just dev`, verify the Tauri window opens with the Vue frontend rendered.

**Acceptance Scenarios**:

1. **Given** a fresh clone of the repository, **When** the developer runs `just setup && just dev`, **Then** all dependencies install and the Tauri development window opens displaying a Vue+PrimeVue placeholder page
2. **Given** the development server is running, **When** the developer edits a Vue component, **Then** the change appears in the Tauri window within 2 seconds (hot-reload)
3. **Given** the development server is running, **When** the developer modifies a Rust source file, **Then** the Tauri backend recompiles and the app restarts automatically

---

### User Story 2 - Run All Quality Checks Locally (Priority: P2)

A developer wants to run the same checks that CI runs, locally, before pushing. They run a single command that executes Rust formatting, linting, tests, and Vue linting and tests. All checks pass on a fresh scaffold with zero code.

**Why this priority**: Local quality checks catch issues before CI, speeding up development. Must match CI to avoid surprises.

**Independent Test**: Run `just check` on a fresh clone — all checks pass with exit code 0.

**Acceptance Scenarios**:

1. **Given** a fresh clone, **When** the developer runs `just check`, **Then** `cargo fmt --check`, `cargo clippy`, `cargo test`, `pnpm lint`, `pnpm test`, and `pnpm build` all pass
2. **Given** a fresh clone, **When** the developer runs `just test`, **Then** Rust tests (including the scaffold smoke test) and Vue tests (Vitest) both pass
3. **Given** an intentionally mis-formatted Rust file, **When** the developer runs `just check`, **Then** `cargo fmt --check` fails with a clear diff

---

### User Story 3 - CI Validates Every Pull Request (Priority: P3)

A contributor opens a pull request. GitHub Actions automatically runs Rust and Vue quality checks across all three workspace crates and the frontend. The PR is blocked from merging until all checks pass and a reviewer approves.

**Why this priority**: CI enforcement prevents regressions and ensures code quality from day one.

**Independent Test**: Open a PR with valid code — CI passes. Open a PR with a clippy warning — CI fails and blocks merge.

**Acceptance Scenarios**:

1. **Given** a PR is opened against `main`, **When** CI triggers, **Then** it runs three parallel jobs: `check-rust` (Ubuntu: fmt, clippy, test), `check-frontend` (Ubuntu: lint, test, build), and `check-windows` (Windows: check, test — only when `crates/**` changed)
2. **Given** CI passes and a reviewer approves, **When** the PR is merged, **Then** the merge completes successfully
3. **Given** CI fails on any check, **When** the contributor views the PR, **Then** the failing step is clearly identified with actionable output

---

### User Story 4 - Automated Dependency Updates (Priority: P4)

The repository receives automated pull requests when Rust crate or npm package updates are available. Each update PR triggers CI to verify compatibility before a human reviews.

**Why this priority**: Keeps dependencies current with minimal manual effort, reducing security risk.

**Independent Test**: Verify Dependabot configuration is valid by checking that `dependabot.yml` parses correctly and covers both Cargo and npm ecosystems.

**Acceptance Scenarios**:

1. **Given** a new version of a Cargo dependency is published, **When** Dependabot runs its scheduled check, **Then** it opens a PR updating `Cargo.lock` with CI triggered
2. **Given** a new version of an npm dependency is published, **When** Dependabot runs its scheduled check, **Then** it opens a PR updating `pnpm-lock.yaml` with CI triggered

---

### Edge Cases

- What happens when a developer builds on macOS or Linux (not Windows)? The workspace MUST compile, but Windows-specific crates are excluded via `cfg(windows)`. The Tauri dev server works on all platforms.
- What happens when pnpm is not installed? `just setup` MUST detect missing tools and print clear installation instructions.
- What happens when the Rust toolchain version is wrong? `rust-toolchain.toml` pins the version, and `rustup` auto-installs it on first build.
- What happens when Tauri system dependencies are missing? `just setup` MUST check for platform-specific prerequisites and print the install command. Required deps: macOS — Xcode CLI tools; Linux — `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`; Windows — Visual Studio Build Tools (C++ workload), WebView2 (pre-installed on Windows 11).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Repository MUST have a Cargo workspace with three member crates: `astro-up-core` (library), `astro-up-cli` (library + binary), `astro-up-gui` (library + binary)
- **FR-002**: `astro-up-core` MUST compile as a library with no binary target and export a public API from `lib.rs`
- **FR-003**: `astro-up-cli` MUST compile as both a library (`lib.rs`) and binary (`main.rs`) with `clap` as a dependency
- **FR-004**: `astro-up-gui` MUST compile as both a library (`lib.rs`) and binary (`main.rs`) with Tauri v2 as a dependency
- **FR-005**: `astro-up-gui` MUST include a `tauri.conf.json` with app identifier `dev.nightwatch.astro-up`, window title `Astro-Up`, and default window size 1024x768
- **FR-006**: Frontend MUST be a Vue 3 application using PrimeVue component library and VueQuery for server state, built with Vite
- **FR-007**: Frontend MUST render a placeholder page containing a PrimeVue Card component with the app name ("Astro-Up"), version (from package.json), and a brief description, confirming the PrimeVue theme and component rendering work
- **FR-008**: GitHub Actions CI MUST run four jobs: (1) `check-rust` on Ubuntu — `cargo fmt --check`, `cargo clippy -p astro-up-core -p astro-up-cli -- -D warnings`, `cargo test -p astro-up-core -p astro-up-cli` (fast path, no Tauri system deps, ~30s); (2) `check-gui` on Ubuntu — `cargo clippy -p astro-up-gui`, `cargo test -p astro-up-gui` with Tauri system deps installed, triggered only when `crates/astro-up-gui/**` changes (path filter via `dorny/paths-filter`); (3) `check-frontend` on Ubuntu — `pnpm install --frozen-lockfile`, `pnpm lint`, `pnpm test`, `pnpm build`; (4) `check-windows` on Windows — `cargo check --workspace`, `cargo test --workspace`, triggered only when `crates/**` changes. All Rust jobs use `Swatinem/rust-cache`. CI MUST validate semantic PR titles via `amannn/action-semantic-pull-request`.
- **FR-009**: Branch protection on `main` MUST require CI passage and at least one PR review before merge
- **FR-010**: Repository MUST include a `CLAUDE.md` documenting project conventions for Rust, Tauri, and Vue development
- **FR-011**: Repository MUST include Dependabot configuration for Cargo, npm, and GitHub Actions ecosystems (weekly schedule, `chore(deps)` / `chore(ci)` commit prefixes, matching nightwatch-astro org convention)
- **FR-012**: Repository MUST include release-plz configuration (`release-plz.toml`) for automated version management, changelog generation, and crates.io publishing. The release workflow MUST delegate to the shared `nightwatch-astro/.github` reusable workflow (`rust-release.yml`). Release-plz MUST use conventional commit parsing with semantic groups (feat, fix, perf, refactor, docs, test, chore), enable git tags, GitHub releases, and dependency updates in release PRs
- **FR-013**: Repository MUST include a `.gitignore` covering Rust (`target/`), Node (`node_modules/`), and Tauri build artifacts
- **FR-014**: Changelog generation MUST be handled by release-plz's built-in git-cliff integration via the `[changelog]` section in `release-plz.toml`. No separate `cliff.toml` is needed
- **FR-015**: Repository MUST include a `Justfile` with recipes for common development tasks: `setup`, `dev`, `build`, `test`, `check`, `fmt`, `lint`
- **FR-016**: `rust-toolchain.toml` MUST pin the Rust edition and toolchain channel
- **FR-017**: Each crate MUST include a smoke test that verifies compilation and basic function availability
- **FR-018**: The workspace MUST compile on macOS, Linux, and Windows, with Windows-specific dependencies gated behind `cfg(windows)` or optional features
- **FR-019**: `astro-up-gui` MUST include a Tauri v2 capabilities file (`capabilities/default.json`) granting core permissions: `core:default`, `window:default`, `app:default`. Additional permissions are added in later specs as features require them

### Key Entities

- **Workspace**: The Cargo workspace root defining member crates and shared dependencies
- **Crate**: A Rust package within the workspace — core (shared logic), cli (command-line interface), gui (desktop application)
- **Frontend**: The Vue 3 application served by the Tauri webview, containing components, composables, and views

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can clone, setup, and launch the development environment in under 5 minutes on a machine with Rust and Node.js pre-installed
- **SC-002**: All three crates compile without warnings on `cargo clippy --workspace`
- **SC-003**: All CI checks pass on the initial scaffold commit with zero manual intervention
- **SC-004**: The Tauri development window opens and displays the Vue+PrimeVue placeholder page
- **SC-005**: Hot-reload reflects frontend changes within 2 seconds during development
- **SC-006**: `just check` completes all quality gates in under 60 seconds on the initial scaffold

## Clarifications

### Session 2026-03-29

- Q: CI platform matrix — which OS runners for which jobs? → A: Three self-contained jobs: `check-rust` (Ubuntu, every PR), `check-frontend` (Ubuntu, every PR), `check-windows` (Windows, only on `crates/**` changes). No artifact sharing between runners. Windows job defined but skipped until Spec 005 lands `cfg(windows)` code. Only Windows-requiring jobs run on Windows.
- Q: Tauri system dependencies per platform? → A: Enumerated in edge cases and CLAUDE.md. macOS: Xcode CLI tools. Linux: WebKit2GTK + build tools. Windows: VS Build Tools + WebView2.
- Q: Tauri v2 capability permissions? → A: FR-019 added — `core:default`, `window:default`, `app:default`. Extended in later specs.
- Q: Placeholder page content? → A: FR-007 clarified — PrimeVue Card with app name, version, description.
- Q: Default window dimensions? → A: FR-005 clarified — 1024x768.
- Q: CI caching strategy? → A: FR-008 clarified — `Swatinem/rust-cache` for Cargo, pnpm store cache for frontend.
- Q: Release automation tool? → A: FR-012 updated — release-plz (not release-please), matching nightwatch-astro org convention. Delegates to shared `rust-release.yml` reusable workflow.

## Assumptions

- Developer machines have Rust (via rustup), Node.js (v22+), and pnpm pre-installed
- The GitHub repository `nightwatch-astro/astro-up` already exists with LICENSE and `.specify/` directory
- Tauri v2 system dependencies (OS-specific build tools) are documented in CLAUDE.md but not auto-installed by `just setup`
- The Cargo workspace uses the 2024 Rust edition
- PrimeVue Aura Dark theme is the default — no theme switching in the scaffold
- No application logic is implemented in this spec — only the project structure and build pipeline
