# Quickstart: Manifest URL Validation & Pipeline Hardening

## Prerequisites

- Rust toolchain (1.85+) with clippy, rustfmt
- Access to `../astro-up-manifests` sibling repo
- MCP Playwright server for interactive validation

## Build & Test

```sh
cd ../astro-up-manifests
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Run Audit

```sh
# Full audit (all manifests)
cargo run -p astro-up-checker -- --audit --manifests manifests --versions versions

# Single manifest
cargo run -p astro-up-checker -- --audit --filter nina-app --manifests manifests --versions versions

# Skip URL validation (faster)
cargo run -p astro-up-checker -- --audit --skip-url-validation --manifests manifests
```

## Fix → Verify Loop

```sh
# 1. Edit the manifest
vim manifests/stellarium-app.toml

# 2. Re-run audit for that manifest
cargo run -p astro-up-checker -- --audit --filter stellarium-app --manifests manifests --versions versions

# 3. Repeat until pass
```

## Compile Catalog

```sh
cargo run -p astro-up-compiler -- --manifests manifests --versions versions --output catalog.db
```

## Validate Manifests Only (no version check)

```sh
cargo run -p astro-up-compiler -- --manifests manifests --validate
```
