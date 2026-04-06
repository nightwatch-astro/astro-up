//! WMI-based installed software enumeration.
//!
//! Queries `Win32_InstalledWin32Program` once to get all installed programs,
//! then matches catalog packages against the results by name/vendor/aliases.
//! This replaces per-package registry key lookups with a single system query.

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::types::Version;

/// A single installed program from WMI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledProgram {
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub program_id: String,
}

/// Result of the WMI enumeration scan.
#[derive(Debug)]
pub struct WmiScanResult {
    pub programs: Vec<InstalledProgram>,
    pub duration: std::time::Duration,
}

/// Query WMI for all installed programs.
///
/// Returns all entries from `Win32_InstalledWin32Program`.
/// This is fast (< 1s) and covers MSI, InnoSetup, NSIS, WiX installs.
#[cfg(windows)]
pub fn enumerate_installed() -> Result<WmiScanResult, String> {
    use std::time::Instant;

    let start = Instant::now();

    let wmi_con = wmi::COMLibrary::new()
        .and_then(|lib| wmi::WMIConnection::new(lib))
        .map_err(|e| format!("WMI connection failed: {e}"))?;

    #[derive(Deserialize, Debug)]
    #[serde(rename = "Win32_InstalledWin32Program")]
    #[serde(rename_all = "PascalCase")]
    struct WmiProgram {
        name: Option<String>,
        version: Option<String>,
        vendor: Option<String>,
        program_id: Option<String>,
    }

    let results: Vec<WmiProgram> = wmi_con
        .query()
        .map_err(|e| format!("WMI query failed: {e}"))?;

    let programs: Vec<InstalledProgram> = results
        .into_iter()
        .filter_map(|p| {
            let name = p.name.filter(|n| !n.is_empty())?;
            Some(InstalledProgram {
                name,
                version: p.version.unwrap_or_default(),
                vendor: p.vendor.unwrap_or_default(),
                program_id: p.program_id.unwrap_or_default(),
            })
        })
        .collect();

    let duration = start.elapsed();
    debug!(
        count = programs.len(),
        duration_ms = duration.as_millis() as u64,
        "WMI enumeration complete"
    );

    Ok(WmiScanResult { programs, duration })
}

#[cfg(not(windows))]
pub fn enumerate_installed() -> Result<WmiScanResult, String> {
    Ok(WmiScanResult {
        programs: Vec::new(),
        duration: std::time::Duration::ZERO,
    })
}

/// Match a catalog package against WMI results.
///
/// Matching strategies (in priority order):
/// 1. Exact `program_id` match (if manifest provides one)
/// 2. Exact name match (case-insensitive)
/// 3. Name contained in WMI name or vice versa (case-insensitive)
/// 4. Alias match against WMI name
pub fn match_package(
    package_name: &str,
    aliases: &[String],
    program_id: Option<&str>,
    wmi_programs: &[InstalledProgram],
) -> Option<MatchedProgram> {
    // 1. Exact program_id match
    if let Some(pid) = program_id {
        if let Some(prog) = wmi_programs.iter().find(|p| p.program_id == pid) {
            debug!(
                package = package_name,
                program_id = pid,
                "matched by program_id"
            );
            return Some(MatchedProgram {
                program: prog.clone(),
                strategy: MatchStrategy::ProgramId,
            });
        }
    }

    let name_lower = package_name.to_lowercase();

    // 2. Exact name match (case-insensitive)
    if let Some(prog) = wmi_programs
        .iter()
        .find(|p| p.name.to_lowercase() == name_lower)
    {
        debug!(
            package = package_name,
            wmi_name = prog.name,
            "matched by exact name"
        );
        return Some(MatchedProgram {
            program: prog.clone(),
            strategy: MatchStrategy::ExactName,
        });
    }

    // 3. Containment match: package name in WMI name or vice versa
    if let Some(prog) = wmi_programs.iter().find(|p| {
        let wmi_lower = p.name.to_lowercase();
        wmi_lower.contains(&name_lower) || name_lower.contains(&wmi_lower)
    }) {
        debug!(
            package = package_name,
            wmi_name = prog.name,
            "matched by name containment"
        );
        return Some(MatchedProgram {
            program: prog.clone(),
            strategy: MatchStrategy::NameContains,
        });
    }

    // 4. Alias match
    for alias in aliases {
        let alias_lower = alias.to_lowercase();
        if let Some(prog) = wmi_programs.iter().find(|p| {
            let wmi_lower = p.name.to_lowercase();
            wmi_lower.contains(&alias_lower) || alias_lower.contains(&wmi_lower)
        }) {
            debug!(
                package = package_name,
                alias,
                wmi_name = prog.name,
                "matched by alias"
            );
            return Some(MatchedProgram {
                program: prog.clone(),
                strategy: MatchStrategy::Alias,
            });
        }
    }

    None
}

/// A matched WMI program with the strategy used.
#[derive(Debug, Clone)]
pub struct MatchedProgram {
    pub program: InstalledProgram,
    pub strategy: MatchStrategy,
}

/// How the match was made (for logging/diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchStrategy {
    ProgramId,
    ExactName,
    NameContains,
    Alias,
}

impl MatchedProgram {
    /// Extract a parsed version from the WMI result.
    pub fn version(&self) -> Option<Version> {
        if self.program.version.is_empty() {
            None
        } else {
            Some(Version::parse(&self.program.version))
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_programs() -> Vec<InstalledProgram> {
        vec![
            InstalledProgram {
                name: "N.I.N.A. - Nighttime Imaging 'N' Astronomy".into(),
                version: "3.1.2.9001".into(),
                vendor: "Stefan Gärtner".into(),
                program_id: "NINA 2_is1".into(),
            },
            InstalledProgram {
                name: "Stellarium 26.1".into(),
                version: "26.1".into(),
                vendor: "Stellarium".into(),
                program_id: "Stellarium_is1".into(),
            },
            InstalledProgram {
                name: "ZWO ASIStudio".into(),
                version: "1.8.0".into(),
                vendor: "ZWO".into(),
                program_id: "ASIStudio".into(),
            },
            InstalledProgram {
                name: "PHD2 Guiding".into(),
                version: "2.6.13".into(),
                vendor: "Open PHD Guiding".into(),
                program_id: "PHDGuiding2_is1".into(),
            },
        ]
    }

    #[test]
    fn match_by_program_id() {
        let programs = test_programs();
        let result = match_package("nina-app", &[], Some("NINA 2_is1"), &programs);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.strategy, MatchStrategy::ProgramId);
        assert_eq!(m.program.name, "N.I.N.A. - Nighttime Imaging 'N' Astronomy");
    }

    #[test]
    fn match_by_exact_name() {
        let programs = test_programs();
        let result = match_package("ZWO ASIStudio", &[], None, &programs);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.strategy, MatchStrategy::ExactName);
    }

    #[test]
    fn match_by_containment() {
        let programs = test_programs();
        let result = match_package("Stellarium", &[], None, &programs);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.strategy, MatchStrategy::NameContains);
        assert!(m.program.name.contains("Stellarium"));
    }

    #[test]
    fn match_by_alias() {
        let programs = test_programs();
        let result = match_package(
            "phd2-app",
            &["PHD2".into(), "PHD Guiding".into()],
            None,
            &programs,
        );
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.strategy, MatchStrategy::Alias);
    }

    #[test]
    fn no_match_returns_none() {
        let programs = test_programs();
        let result = match_package("nonexistent-app", &[], None, &programs);
        assert!(result.is_none());
    }

    #[test]
    fn version_parsing() {
        let m = MatchedProgram {
            program: InstalledProgram {
                name: "Test".into(),
                version: "3.1.2".into(),
                vendor: String::new(),
                program_id: String::new(),
            },
            strategy: MatchStrategy::ExactName,
        };
        assert_eq!(m.version().unwrap().to_string(), "3.1.2");
    }

    #[test]
    fn empty_version() {
        let m = MatchedProgram {
            program: InstalledProgram {
                name: "Test".into(),
                version: String::new(),
                vendor: String::new(),
                program_id: String::new(),
            },
            strategy: MatchStrategy::ExactName,
        };
        assert!(m.version().is_none());
    }
}
