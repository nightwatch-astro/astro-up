# Research: Repository Scaffolding

## Tauri v2 + Cargo Workspace Integration

**Decision**: Manual project setup instead of `cargo tauri init`.

**Rationale**: `cargo tauri init` creates a `src-tauri/` directory structure. Our workspace uses `crates/astro-up-gui/` instead. Manual setup gives full control over the directory layout and avoids post-init restructuring.

**Key findings**:
- `tauri.conf.json` must live in the same directory as the Tauri crate's `Cargo.toml`
- `build.frontendDist` and `build.devUrl` are relative to `tauri.conf.json` location
- `build.rs` must call `tauri_build::build()` — required for Tauri compilation
- Tauri v2 uses a capabilities system (`capabilities/default.json`) instead of v1's allowlist
- The `@tauri-apps/cli` package provides `cargo tauri dev` and `cargo tauri build` commands

**Alternatives considered**:
- `cargo tauri init` then move files: fragile, path references break
- `src-tauri/` as workspace member: non-standard naming, confusing with 3 crates

## PrimeVue 4 Theme Configuration

**Decision**: Aura preset with system dark mode detection.

**Rationale**: Aura is PrimeVue's default theme. `darkModeSelector: 'system'` follows OS preference without custom toggle logic. No CSS layers needed since PrimeVue is the only component library.

**Key findings**:
- Themes imported from `@primeuix/themes/aura` (not the old `primevue/themes` path)
- `cssLayer: false` avoids CSS specificity issues when no other libraries compete
- `prefix: 'p'` is default — PrimeVue classes use `p-` prefix
- No additional CSS imports needed — styled mode includes all component styles

## VueQuery (TanStack Query for Vue)

**Decision**: `@tanstack/vue-query` v5 with default configuration.

**Rationale**: VueQuery manages server state (caching, background refetching, stale-while-revalidate). In this scaffold, we only install and configure the plugin. Composables wrapping `invoke()` are added in Spec 016 (Vue Frontend).

**Key findings**:
- Setup: `app.use(VueQueryPlugin)` — one line, no custom QueryClient needed for scaffold
- DevTools: `@tanstack/vue-query-devtools` available but deferred (not needed in scaffold)
- SSR/hydration: not applicable (Tauri is a desktop app, no SSR)

## CI Platform Strategy

**Decision**: 3-job split — Ubuntu for fast checks, Windows only for Rust compilation.

**Rationale**: Ubuntu runners start faster and are more predictable. Frontend checks are platform-independent. Windows CI only adds value when `cfg(windows)` code exists (Spec 005+).

**Key findings**:
- GitHub Actions public repos: unlimited minutes, but Windows/macOS runners are 2x/10x slower
- Path filters (`on.push.paths`) can scope Windows job to `crates/**` changes only
- No artifact sharing needed — each job is self-contained
- `cargo check` on Windows catches `cfg(windows)` compilation errors that Ubuntu misses

## Release-Please Configuration

**Decision**: Rust release type with Cargo.toml version management.

**Rationale**: release-please reads conventional commits, generates changelogs, and creates release PRs. The Rust release type updates `Cargo.toml` versions. Combined with git-cliff for detailed changelogs.

**Alternatives considered**:
- cargo-release: more Rust-native but less GitHub-integrated
- manual releases: error-prone, doesn't scale
