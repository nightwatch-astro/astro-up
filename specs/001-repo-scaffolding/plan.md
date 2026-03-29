# Implementation Plan: Repository Scaffolding

**Branch**: `001-repo-scaffolding` | **Date**: 2026-03-29 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/001-repo-scaffolding/spec.md`

## Summary

Scaffold the `nightwatch-astro/astro-up` repository with a Cargo workspace (3 crates), Tauri v2 desktop app, Vue 3 + PrimeVue frontend, GitHub Actions CI (3-job split), and developer tooling (Justfile, Dependabot, release-please, git-cliff).

## Technical Context

**Language/Version**: Rust 2024 edition (stable channel, pinned via `rust-toolchain.toml`)
**Primary Dependencies**: Tauri v2, clap 4, Vue 3, PrimeVue 4, @tanstack/vue-query 5, Vite 6
**Storage**: N/A (scaffold only)
**Testing**: `cargo test` (Rust), Vitest (Vue), `insta` (snapshot testing, dev-dependency)
**Target Platform**: Windows (primary), macOS/Linux (cross-platform compilation)
**Project Type**: Desktop application (Tauri) + CLI tool
**Constraints**: Zero `cfg(windows)` code in this spec — Windows-gated code arrives in Spec 005

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Modules-First Crate Layout | PASS | 3 crates, `astro-up-core` has module stubs |
| II. Platform Awareness | PASS | `cfg(windows)` in workspace Cargo.toml for future optional deps |
| III. Test-First | PASS | Smoke tests in each crate, Vitest for frontend |
| IV. Thin Tauri Boundary | PASS | `astro-up-gui/lib.rs` delegates to core (placeholder) |
| V. Spec-Driven | PASS | This plan follows speckit workflow |
| VI. Simplicity | PASS | Minimal scaffold, no premature abstractions |

## Project Structure

### Documentation (this feature)

```text
specs/001-repo-scaffolding/
├── spec.md
├── plan.md              # This file
├── research.md          # Phase 0 output
├── quickstart.md        # Phase 1 output
├── checklists/
│   └── requirements.md
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
nightwatch-astro/astro-up/
├── .github/
│   ├── workflows/
│   │   └── ci.yml                    # 3-job CI: check-rust, check-frontend, check-windows
│   ├── dependabot.yml                # Cargo + npm
│   └── release-please.yml            # Automated releases
├── .specify/                         # Already exists
├── crates/
│   ├── astro-up-core/
│   │   ├── Cargo.toml                # lib crate
│   │   └── src/
│   │       └── lib.rs                # Public API stub + smoke test
│   ├── astro-up-cli/
│   │   ├── Cargo.toml                # lib+bin, depends on core + clap
│   │   └── src/
│   │       ├── lib.rs                # CLI logic (testable)
│   │       └── main.rs               # Entry point
│   └── astro-up-gui/
│       ├── Cargo.toml                # lib+bin, depends on core + tauri
│       ├── src/
│       │   ├── lib.rs                # Tauri commands (testable)
│       │   └── main.rs               # Tauri app init
│       ├── tauri.conf.json           # App config
│       ├── build.rs                  # Tauri build script
│       ├── icons/                    # Default Tauri icons
│       └── capabilities/
│           └── default.json          # Tauri v2 capability permissions
├── frontend/
│   ├── src/
│   │   ├── App.vue                   # Root component with PrimeVue
│   │   ├── main.ts                   # Vue app init + PrimeVue + VueQuery
│   │   ├── styles.css                # Minimal global styles
│   │   └── vite-env.d.ts             # Vite type declarations
│   ├── index.html                    # Vite entry point
│   ├── package.json                  # Vue, PrimeVue, VueQuery, Vitest
│   ├── pnpm-lock.yaml
│   ├── vite.config.ts                # Vite config with Tauri host
│   ├── tsconfig.json                 # TypeScript config
│   ├── tsconfig.node.json            # Node-side TS config
│   └── eslint.config.js              # ESLint flat config
├── research/                         # Already exists
├── specs/                            # Feature specs
├── Cargo.toml                        # Workspace root
├── Cargo.lock
├── CLAUDE.md                         # Project conventions
├── LICENSE                           # Already exists (Apache-2.0)
├── Justfile                          # Dev recipes
├── cliff.toml                        # git-cliff changelog config
├── rust-toolchain.toml               # Pinned Rust toolchain
└── .gitignore                        # Rust + Node + Tauri
```

**Structure Decision**: Cargo workspace with `crates/` directory for Rust packages, `frontend/` for Vue app. Tauri v2 convention places `tauri.conf.json` inside the GUI crate (not `src-tauri/` — we use `crates/astro-up-gui/` instead).

## Key Technical Decisions

### Tauri v2 integration with Cargo workspace

Tauri v2 expects its Rust code in a directory containing `tauri.conf.json`. By default this is `src-tauri/`, but we use `crates/astro-up-gui/`. The `tauri.conf.json` must set:
- `build.frontendDist`: `../../frontend/dist`
- `build.devUrl`: `http://localhost:5173`
- `build.beforeDevCommand`: `pnpm --dir ../../frontend dev`
- `build.beforeBuildCommand`: `pnpm --dir ../../frontend build`

`cargo tauri init` is NOT used — we create the structure manually to integrate with the workspace layout.

### PrimeVue setup

```javascript
import PrimeVue from 'primevue/config';
import Aura from '@primeuix/themes/aura';

app.use(PrimeVue, {
    theme: {
        preset: Aura,
        options: {
            darkModeSelector: 'system',
            cssLayer: false
        }
    }
});
```

Dark mode follows system preference via `darkModeSelector: 'system'`.

### VueQuery setup

```javascript
import { VueQueryPlugin } from '@tanstack/vue-query';
app.use(VueQueryPlugin);
```

Composables wrapping Tauri `invoke()` calls will be added in later specs.

### CI workflow structure

Three parallel jobs in `ci.yml`:
1. **check-rust** (ubuntu-latest): `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
2. **check-frontend** (ubuntu-latest): `pnpm install --frozen-lockfile`, `pnpm lint`, `pnpm test`, `pnpm build`
3. **check-windows** (windows-latest): `cargo check --workspace`, `cargo test --workspace` — path filter on `crates/**`, `Cargo.toml`, `Cargo.lock`

### Justfile recipes

| Recipe | Command |
|--------|---------|
| `setup` | Install frontend deps, verify toolchain |
| `dev` | `cargo tauri dev` (starts Vite + Tauri) |
| `build` | `cargo tauri build` |
| `test` | `cargo test --workspace && pnpm --dir frontend test` |
| `check` | fmt + clippy + test + lint + frontend build |
| `fmt` | `cargo fmt --all` |
| `lint` | `cargo clippy --workspace -- -D warnings && pnpm --dir frontend lint` |

## Dependencies

### Rust (Cargo.toml workspace)

| Crate | Version | Location |
|-------|---------|----------|
| clap | 4 (derive) | astro-up-cli |
| tauri | 2 | astro-up-gui |
| tauri-build | 2 | astro-up-gui (build-dep) |
| serde | 1 (derive) | workspace |
| serde_json | 1 | workspace |
| insta | 1 | workspace (dev-dep) |

### Frontend (package.json)

| Package | Version | Purpose |
|---------|---------|---------|
| vue | ^3 | UI framework |
| primevue | ^4 | Component library |
| @primeuix/themes | ^1 | PrimeVue theme presets |
| @tanstack/vue-query | ^5 | Server state management |
| @tauri-apps/api | ^2 | Tauri IPC bridge |
| typescript | ^5 | Type safety |
| vite | ^6 | Build tool |
| @vitejs/plugin-vue | ^5 | Vite Vue plugin |
| vitest | ^3 | Test runner |
| eslint | ^9 | Linter |
