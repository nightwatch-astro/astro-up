// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Catalog module — fetch, verify, and query the software catalog.

pub mod fetch;
pub mod lock;
pub mod manager;
pub mod manifest;
pub mod reader;
pub mod sidecar;
pub mod types;
pub mod verify;

pub use manager::CatalogManager;
pub use reader::SqliteCatalogReader;
pub use sidecar::CatalogSidecar;
pub use types::{
    CatalogFilter, CatalogMeta, FetchResult, PackageId, PackageSummary, SearchResult, VersionEntry,
};
