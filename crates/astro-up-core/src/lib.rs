//! Shared library for astro-up — types, detection, download, install, and orchestration engine.

pub mod adapters;
pub mod backup;
pub mod catalog;
pub mod config;
pub mod detect;
pub mod download;
pub mod engine;
pub mod error;
pub mod events;
pub mod install;
pub mod ledger;
pub mod lifecycle;
pub mod metrics;
pub mod release;
pub mod traits;
pub mod types;

// Re-export rusqlite for consumers that need direct Connection access (orchestrator).
pub use rusqlite;

/// Crate name for runtime identification (user-agent strings, log prefixes).
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");

/// Returns the version of astro-up-core.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Encode a byte slice as lowercase hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}
