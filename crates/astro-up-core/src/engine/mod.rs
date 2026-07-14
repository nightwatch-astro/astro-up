// Copyright (C) 2024-2026 Sjors Robroek
// SPDX-License-Identifier: AGPL-3.0-only

//! Install orchestration engine — coordinates the update pipeline.

pub mod history;
pub mod lock;
pub mod orchestrator;
pub mod planner;
pub mod policy;
pub mod process;
pub mod version_cmp;
