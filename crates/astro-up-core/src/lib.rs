pub mod types;

/// Returns the version of astro-up-core.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
