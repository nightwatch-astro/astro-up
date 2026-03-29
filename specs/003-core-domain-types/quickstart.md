# Quickstart: Core Domain Types

## Using the types

```rust
use astro_up_core::types::{Software, Category, SoftwareType, DetectionMethod};
use astro_up_core::error::CoreError;
use astro_up_core::events::Event;
use astro_up_core::traits::Detector;
use astro_up_core::version::Version;

// Deserialize a manifest
let toml_str = std::fs::read_to_string("manifests/capture/nina.toml")?;
let software: Software = toml::from_str(&toml_str)?;

assert_eq!(software.category, Category::Capture);
assert_eq!(software.software_type, SoftwareType::Application);

// Parse a version (lenient)
let v = Version::parse("3.1.2.3001");
assert_eq!(v.raw, "3.1.2.3001");
assert!(v.parsed.is_some()); // coerced to 3.1.2

// Enum string conversion
use std::str::FromStr;
let cat = Category::from_str("capture").unwrap();
assert_eq!(cat.to_string(), "capture");

// Error handling
let err = CoreError::InstallerFailed {
    exit_code: 1,
    response: KnownExitCode::PackageInUse,
};
println!("{err}"); // "installer failed with exit code 1: package in use"
```

## Running tests

```sh
cargo test -p astro-up-core
```

## Module layout

```
crates/astro-up-core/src/
├── lib.rs          # pub mod declarations + re-exports
├── types/          # All data structs and enums
├── error.rs        # CoreError (thiserror)
├── traits.rs       # Detector, Provider, Installer, etc. (trait_variant)
├── events.rs       # Event enum (serde adjacently tagged)
├── ledger.rs       # LedgerEntry for manual tracking
├── release.rs      # Release struct
├── version.rs      # Lenient Version wrapper
└── metrics.rs      # Metric name constants
```
