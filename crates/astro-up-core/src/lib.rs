pub mod catalog;
pub mod config;
pub mod error;
pub mod events;
pub mod ledger;
pub mod metrics;
pub mod release;
pub mod traits;
pub mod types;

/// Returns the version of astro-up-core.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
