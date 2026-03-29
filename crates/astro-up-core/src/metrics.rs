//! Metric name constants for the `metrics` crate facade.
//!
//! Core types export names only — the engine and UI layers register
//! recorders and emit values using these constants.

pub const DOWNLOAD_BYTES_TOTAL: &str = "astro_up_download_bytes_total";
pub const SCAN_DURATION_SECONDS: &str = "astro_up_scan_duration_seconds";
pub const CHECK_REQUESTS_TOTAL: &str = "astro_up_check_requests_total";
pub const INSTALL_DURATION_SECONDS: &str = "astro_up_install_duration_seconds";
pub const CACHE_HIT_TOTAL: &str = "astro_up_cache_hit_total";
pub const CACHE_MISS_TOTAL: &str = "astro_up_cache_miss_total";
