# Quickstart: 006-registry-pe-detection

## New dependencies

```toml
# crates/astro-up-core/Cargo.toml

[dependencies]
pelite = "0.10"                              # PE file parsing (cross-platform)

[target.'cfg(windows)'.dependencies]
winreg = "0.56"                              # Windows registry access
wmi = "0.14"                                 # WMI queries (driver + hardware)
```

## Module layout

```
crates/astro-up-core/src/
├── detect/
│   ├── mod.rs          # DetectionService impl, chain runner, scan orchestration
│   ├── registry.rs     # cfg(windows) — registry detection method
│   ├── pe.rs           # PE file version extraction (cross-platform)
│   ├── wmi_driver.rs   # cfg(windows) — WMI driver detection
│   ├── ascom.rs        # cfg(windows) — ASCOM Profile detection
│   ├── file.rs         # file_exists + config_file methods
│   ├── cache.rs        # In-memory detection cache
│   ├── hardware.rs     # cfg(windows) — VID:PID hardware discovery
│   └── path.rs         # Path token resolver (shared utility)
├── types/
│   └── detection.rs    # Already exists — DetectionConfig, DetectionMethod
└── lib.rs              # Add `pub mod detect;`
```

## Key patterns

### Detection chain execution
```rust
// Pseudocode — execute fallback chain, stop at first success
async fn run_chain(config: &DetectionConfig, resolver: &PathResolver) -> DetectionResult {
    let result = match config.method {
        DetectionMethod::Registry => registry::detect(config).await,
        DetectionMethod::PeFile => pe::detect(config, resolver).await,
        // ... other methods
    };
    match result {
        DetectionResult::Installed { .. } | DetectionResult::InstalledUnknownVersion { .. } => result,
        _ => match &config.fallback {
            Some(next) => run_chain(next, resolver).await,
            None => result,
        },
    }
}
```

### Platform gating
```rust
// Each Windows-only module:
#[cfg(windows)]
pub async fn detect(config: &DetectionConfig) -> DetectionResult { /* real impl */ }

#[cfg(not(windows))]
pub async fn detect(_config: &DetectionConfig) -> DetectionResult {
    DetectionResult::Unavailable { reason: "Windows-only detection method".into() }
}
```

## Testing approach

- **Unit tests**: Version parsing, path token expansion, VID:PID pattern matching, chain logic
- **Integration tests**: PE file reading with real test binaries (cross-platform)
- **Snapshot tests**: `insta` for scan result serialization
- **Windows-only tests**: Registry and WMI (run on Windows CI job)
- **Test fixtures**: Include a small PE file with known version info in `tests/fixtures/`
