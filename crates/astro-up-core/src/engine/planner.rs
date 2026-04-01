//! Update planner — version comparison, dependency resolution, and plan building.

use serde::{Deserialize, Serialize};

use crate::catalog::PackageId;
use crate::types::{PolicyLevel, Software, Version};

use super::version_cmp::VersionFormat;

// ---------------------------------------------------------------------------
// PackageStatus (forward-declared stub — will move to a shared location once
// the detection/ledger modules define a canonical version)
// ---------------------------------------------------------------------------

/// Current status of a package in the user's environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageStatus {
    /// Package is installed and managed.
    Installed,
    /// Package was detected but not yet acknowledged by the user.
    Detected,
    /// Package is disabled (opted-out of updates).
    Disabled,
    /// Package is not present on the system.
    NotInstalled,
}

// ---------------------------------------------------------------------------
// SkipReason
// ---------------------------------------------------------------------------

/// Why a package was excluded from the update plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SkipReason {
    /// The installed version is already at or above the catalog target.
    UpToDate,
    /// The installed version is newer than anything in the catalog.
    NewerThanCatalog,
    /// A policy rule prevented the update.
    PolicyBlocked {
        /// The policy level that blocked the update.
        policy: PolicyLevel,
    },
    /// The package is configured for manual-only updates.
    ManualOnly,
    /// The package is disabled.
    Disabled,
    /// A dependency of this package failed or was skipped.
    DependencyFailed {
        /// The dependency that caused the skip.
        dep_id: PackageId,
    },
}

// ---------------------------------------------------------------------------
// SkippedPackage
// ---------------------------------------------------------------------------

/// A package that was evaluated but excluded from the update plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedPackage {
    /// Identifier of the skipped package.
    pub package_id: PackageId,
    /// Why the package was skipped.
    pub reason: SkipReason,
    /// Current status of the package at plan time.
    pub status: PackageStatus,
}

// ---------------------------------------------------------------------------
// PlannedUpdate
// ---------------------------------------------------------------------------

/// A single package update that the engine intends to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedUpdate {
    /// Identifier of the package to update.
    pub package_id: PackageId,
    /// Full software definition from the catalog.
    pub software: Software,
    /// Currently installed version.
    pub current_version: Version,
    /// Version the engine will update to.
    pub target_version: Version,
    /// Catalog entry for the target version (download URLs, hashes, etc.).
    pub version_entry: crate::catalog::VersionEntry,
    /// How the version string should be parsed and compared.
    pub version_format: VersionFormat,
    /// Whether a backup configuration exists for this package.
    pub has_backup_config: bool,
    /// Other packages that must be updated before this one.
    pub dependencies: Vec<PackageId>,
}

// ---------------------------------------------------------------------------
// UpdatePlan
// ---------------------------------------------------------------------------

/// The complete update plan produced by the planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlan {
    /// Packages that will be updated, in dependency order.
    pub items: Vec<PlannedUpdate>,
    /// Packages that were evaluated but excluded.
    pub skipped: Vec<SkippedPackage>,
    /// Non-fatal warnings encountered during planning.
    pub warnings: Vec<String>,
}
