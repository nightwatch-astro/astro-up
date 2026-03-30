//! Detection system public API contract.
//!
//! This file defines the trait boundaries for spec 006.
//! Implementation lives in `crates/astro-up-core/src/detect/`.

use crate::types::{PackageId, Version};
use crate::catalog::PackageSummary;
use crate::ledger::LedgerEntry;
use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};

// -- Core detection result --

#[derive(Debug, Clone, PartialEq)]
pub enum DetectionResult {
    Installed { version: Version, method: DetectionMethod },
    InstalledUnknownVersion { method: DetectionMethod },
    NotInstalled,
    Unavailable { reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DetectionMethod {
    Registry,
    PeFile,
    Wmi,
    DriverStore,
    AscomProfile,
    FileExists,
    ConfigFile,
}

// -- Scan results --

pub struct ScanResult {
    pub results: Vec<PackageDetection>,
    pub errors: Vec<ScanError>,
    pub duration: Duration,
    pub scanned_at: DateTime<Utc>,
}

pub struct PackageDetection {
    pub package_id: PackageId,
    pub result: DetectionResult,
}

pub struct ScanError {
    pub package_id: PackageId,
    pub method: DetectionMethod,
    pub error: String,
}

// -- Hardware discovery --

pub struct HardwareMatch {
    pub vid_pid: VidPid,
    pub device_name: String,
    pub suggested_package: PackageId,
    pub already_managed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VidPid {
    pub vendor_id: u16,
    pub product_id: Option<u16>, // None = wildcard
}

// -- Public API trait --

/// Main detection interface. Consumers (CLI, GUI) use this trait.
#[trait_variant::make(Send)]
pub trait DetectionService {
    /// Run a full scan across all catalog packages.
    /// Returns results for all packages + non-fatal errors.
    /// Persists Acknowledged ledger entries for newly detected packages.
    /// Removes Acknowledged entries for packages no longer detected.
    async fn scan(&self) -> Result<ScanResult, DetectionError>;

    /// Get cached result for a single package, or None if not cached.
    fn cached(&self, id: &PackageId) -> Option<&DetectionResult>;

    /// Invalidate cache for a specific package (after install/update).
    fn invalidate(&self, id: &PackageId);

    /// Invalidate entire cache (explicit scan request).
    fn invalidate_all(&self);

    /// Discover connected hardware and suggest unmanaged driver packages.
    async fn discover_hardware(&self) -> Result<Vec<HardwareMatch>, DetectionError>;
}

// -- Path resolution --

/// Expands platform tokens in manifest file paths.
pub trait PathResolver {
    /// Expand tokens like `{program_files}` to actual paths.
    /// Returns None if the token is not available on this platform.
    fn expand(&self, template: &str) -> Option<String>;
}

// -- Error type --

#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("catalog unavailable: {0}")]
    CatalogError(String),

    #[error("ledger error: {0}")]
    LedgerError(String),

    #[error("WMI connection failed: {0}")]
    WmiConnectionError(String),
}
