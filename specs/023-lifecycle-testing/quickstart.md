# Quickstart: 023-lifecycle-testing

## Single Package Test

```bash
# Clone manifests repo
git clone https://github.com/nightwatch-astro/astro-up-manifests /tmp/manifests

# Run lifecycle test for NINA
astro-up lifecycle-test nina-app --manifest-path /tmp/manifests

# With specific version
astro-up lifecycle-test nina-app --manifest-path /tmp/manifests --version 3.1.2

# Dry-run (download + probe only, no install/uninstall)
astro-up lifecycle-test nina-app --manifest-path /tmp/manifests --dry-run

# JSON output for scripting
astro-up lifecycle-test nina-app --manifest-path /tmp/manifests --json

# download_only package (requires --install-dir)
astro-up lifecycle-test zwo-firmware --manifest-path /tmp/manifests --install-dir C:\Temp\firmware
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All phases passed |
| 1 | Install failed |
| 2 | Discovery failed (installed but no detection found) |
| 3 | Uninstall or verification failed |
| 4 | Download failed |

## GitHub Actions Workflow

### Single package (manual)
1. Go to Actions -> "Lifecycle Test"
2. Click "Run workflow"
3. Enter package ID (e.g., `nina-app`)
4. Optionally: version, dry-run checkbox
5. Review job summary for results
6. If detection found: PR auto-created against manifests repo

### Matrix sweep

The matrix sweep (testing all packages missing `[detection]`) is available when
the workflow is called via `workflow_call` or `repository_dispatch` without a
`package_id` input. The `workflow_dispatch` UI requires a package ID.

To trigger a sweep programmatically:

```bash
gh api repos/nightwatch-astro/astro-up/dispatches \
  -f event_type=lifecycle-sweep
```

When triggered without a package ID, the workflow scans manifests for packages
missing `[detection]` and runs each as a parallel job (max 5 concurrent).
Individual PRs are created for each discovered config.

## Key Files

| File | Purpose |
|------|---------|
| `crates/astro-up-core/src/detect/discovery.rs` | Blind detection probing |
| `crates/astro-up-core/src/lifecycle.rs` | Lifecycle test runner |
| `crates/astro-up-core/src/catalog/manifest.rs` | TOML manifest reader |
| `crates/astro-up-cli/src/commands/lifecycle_test.rs` | CLI subcommand |
| `.github/workflows/lifecycle-test.yml` | GitHub Actions workflow |
