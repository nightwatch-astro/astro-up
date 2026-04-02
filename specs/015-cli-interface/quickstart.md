# Quickstart: CLI Interface

## Validate with

```bash
# Build
cargo build -p astro-up-cli

# Run help
cargo run -p astro-up-cli -- --help
cargo run -p astro-up-cli -- show --help

# Show (requires catalog + scan cache)
cargo run -p astro-up-cli -- show
cargo run -p astro-up-cli -- show installed
cargo run -p astro-up-cli -- show --json

# Scan
cargo run -p astro-up-cli -- scan
cargo run -p astro-up-cli -- scan --json

# Search
cargo run -p astro-up-cli -- search nina

# JSON output validation
cargo run -p astro-up-cli -- show --json | jq .
cargo run -p astro-up-cli -- scan --json | jq .

# Verbose logging
cargo run -p astro-up-cli -- --verbose show

# Check log file was created
ls -la ~/.local/share/astro-up/logs/

# Integration tests
cargo test -p astro-up-cli
```

## Key checkpoints

1. `astro-up --help` shows all subcommands with descriptions
2. `astro-up show` renders a styled table (or triggers first-run bootstrap)
3. `astro-up show --json` outputs valid JSON
4. `astro-up scan` runs detection and shows results
5. Log file created at `{data_dir}/logs/astro-up-{date}.log` with JSON entries
6. Ctrl+C during `update` cancels gracefully with exit code 2
7. `astro-up update --all` shows plan table and asks for confirmation
