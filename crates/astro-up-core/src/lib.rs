pub mod backup;
pub mod catalog;
pub mod config;
pub mod detect;
pub mod download;
pub mod error;
pub mod events;
pub mod install;
pub mod ledger;
pub mod metrics;
pub mod release;
pub mod traits;
pub mod types;

/// Returns the version of astro-up-core.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
