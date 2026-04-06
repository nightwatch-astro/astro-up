# Astro-Up development recipes

# Install frontend dependencies and verify toolchain
setup:
    @echo "Checking Rust toolchain..."
    rustc --version
    @echo "Checking pnpm..."
    pnpm --version || (echo "pnpm not found. Install: npm install -g pnpm" && exit 1)
    @echo "Installing frontend dependencies..."
    cd frontend && pnpm install
    @echo "Checking Tauri system dependencies..."
    @echo "  macOS: xcode-select --install"
    @echo "  Linux: sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev"
    @echo "  Windows: Visual Studio Build Tools (C++ workload) + WebView2"
    @echo "Setup complete."

# Start Tauri dev server with Vue hot-reload
dev:
    cargo tauri dev

# Build production Tauri application
build:
    cargo tauri build

# Run all tests (Rust + Vue)
test:
    cargo test --workspace
    cd frontend && pnpm test

# Run all quality checks (matches CI)
check: fmt-check lint test
    cd frontend && pnpm build

# Format all Rust code
fmt:
    cargo fmt --all

# Check Rust formatting (no changes)
fmt-check:
    cargo fmt --all -- --check

# Build CLI and place as sidecar binary for Tauri
build-sidecar:
    cargo build --release -p astro-up-cli
    mkdir -p crates/astro-up-gui/binaries
    @TARGET=$(rustc -vV | sed -n 's/host: //p') && \
      EXT="" && \
      case "$TARGET" in *windows*) EXT=".exe" ;; esac && \
      SRC="target/release/astro-up${EXT}" && \
      DEST="crates/astro-up-gui/binaries/astro-up-cli-${TARGET}${EXT}" && \
      cp "${SRC}" "${DEST}" && \
      echo "Sidecar binary: ${DEST}"

# Run linters (Rust clippy + Vue ESLint)
lint:
    cargo clippy --workspace -- -D warnings
    cd frontend && pnpm lint
