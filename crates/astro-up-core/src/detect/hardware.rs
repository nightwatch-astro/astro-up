use std::fmt;

use serde::{Deserialize, Serialize};

/// USB Vendor ID : Product ID pair for hardware matching.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VidPid {
    pub vendor_id: u16,
    /// None = wildcard (matches any product under this vendor).
    pub product_id: Option<u16>,
}

impl VidPid {
    /// Parse a VID:PID string like "03C3:120A" or "03C3:*".
    pub fn parse(s: &str) -> Option<Self> {
        let (vid_str, pid_str) = s.split_once(':')?;
        let vendor_id = u16::from_str_radix(vid_str.trim(), 16).ok()?;
        let product_id = if pid_str.trim() == "*" {
            None
        } else {
            Some(u16::from_str_radix(pid_str.trim(), 16).ok()?)
        };
        Some(Self {
            vendor_id,
            product_id,
        })
    }

    /// Check if this pattern matches a device's VID:PID.
    pub fn matches(&self, device: &VidPid) -> bool {
        if self.vendor_id != device.vendor_id {
            return false;
        }
        match self.product_id {
            None => true, // wildcard
            Some(pid) => device.product_id == Some(pid),
        }
    }
}

impl fmt::Display for VidPid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.product_id {
            Some(pid) => write!(f, "{:04X}:{:04X}", self.vendor_id, pid),
            None => write!(f, "{:04X}:*", self.vendor_id),
        }
    }
}

/// Result of hardware discovery — a suggested package for connected hardware.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareMatch {
    pub vid_pid: VidPid,
    pub device_name: String,
    pub suggested_package: String,
    pub already_managed: bool,
}

/// Discover connected USB hardware and match against manifest VID:PID patterns.
///
/// Returns matches for devices that have a matching manifest entry.
/// On non-Windows, returns an empty vec.
#[cfg(windows)]
pub async fn discover(
    manifest_patterns: &[(VidPid, String)],
    managed_packages: &std::collections::HashSet<String>,
) -> Vec<HardwareMatch> {
    use serde::Deserialize as _;
    use std::time::Duration;

    #[derive(serde::Deserialize, Debug)]
    #[allow(non_snake_case)]
    struct PnPEntity {
        DeviceID: Option<String>,
        Name: Option<String>,
    }

    let result = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::task::spawn_blocking(|| {
            let com = wmi::COMLibrary::new().map_err(|e| format!("{e}"))?;
            let con = wmi::WMIConnection::new(com).map_err(|e| format!("{e}"))?;
            let devices: Vec<PnPEntity> = con
                .raw_query(
                    "SELECT DeviceID, Name FROM Win32_PnPEntity WHERE DeviceID LIKE 'USB\\\\VID_%'",
                )
                .map_err(|e| format!("{e}"))?;
            Ok::<_, String>(devices)
        })
        .await
        .map_err(|e| format!("{e}"))?
    })
    .await;

    let devices = match result {
        Ok(Ok(d)) => d,
        _ => return Vec::new(),
    };

    let mut matches = Vec::new();
    for device in &devices {
        let Some(ref device_id) = device.DeviceID else {
            continue;
        };
        let Some(device_vidpid) = parse_device_id_vidpid(device_id) else {
            continue;
        };

        for (pattern, package_id) in manifest_patterns {
            if pattern.matches(&device_vidpid) {
                matches.push(HardwareMatch {
                    vid_pid: device_vidpid.clone(),
                    device_name: device.Name.clone().unwrap_or_default(),
                    suggested_package: package_id.clone(),
                    already_managed: managed_packages.contains(package_id),
                });
                break; // one match per device
            }
        }
    }

    matches
}

#[cfg(not(windows))]
pub async fn discover(
    _manifest_patterns: &[(VidPid, String)],
    _managed_packages: &std::collections::HashSet<String>,
) -> Vec<HardwareMatch> {
    Vec::new()
}

/// Parse VID:PID from a Windows DeviceID string like "USB\VID_03C3&PID_120A\..."
#[cfg(any(windows, test))]
fn parse_device_id_vidpid(device_id: &str) -> Option<VidPid> {
    let upper = device_id.to_uppercase();
    let vid_pos = upper.find("VID_")?;
    let vid_str = upper.get(vid_pos + 4..vid_pos + 8)?;
    let vendor_id = u16::from_str_radix(vid_str, 16).ok()?;

    let pid_pos = upper.find("PID_")?;
    let pid_str = upper.get(pid_pos + 4..pid_pos + 8)?;
    let product_id = u16::from_str_radix(pid_str, 16).ok()?;

    Some(VidPid {
        vendor_id,
        product_id: Some(product_id),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_exact_vidpid() {
        let vp = VidPid::parse("03C3:120A").unwrap();
        assert_eq!(vp.vendor_id, 0x03C3);
        assert_eq!(vp.product_id, Some(0x120A));
    }

    #[test]
    fn parse_wildcard_vidpid() {
        let vp = VidPid::parse("03C3:*").unwrap();
        assert_eq!(vp.vendor_id, 0x03C3);
        assert_eq!(vp.product_id, None);
    }

    #[test]
    fn parse_invalid_returns_none() {
        assert!(VidPid::parse("not-a-vidpid").is_none());
        assert!(VidPid::parse("ZZZZ:1234").is_none());
        assert!(VidPid::parse("").is_none());
    }

    #[test]
    fn exact_match() {
        let pattern = VidPid::parse("03C3:120A").unwrap();
        let device = VidPid {
            vendor_id: 0x03C3,
            product_id: Some(0x120A),
        };
        assert!(pattern.matches(&device));
    }

    #[test]
    fn wildcard_match() {
        let pattern = VidPid::parse("03C3:*").unwrap();
        let device = VidPid {
            vendor_id: 0x03C3,
            product_id: Some(0x120A),
        };
        assert!(pattern.matches(&device));
    }

    #[test]
    fn no_match_different_vendor() {
        let pattern = VidPid::parse("03C3:120A").unwrap();
        let device = VidPid {
            vendor_id: 0x1234,
            product_id: Some(0x120A),
        };
        assert!(!pattern.matches(&device));
    }

    #[test]
    fn no_match_different_product() {
        let pattern = VidPid::parse("03C3:120A").unwrap();
        let device = VidPid {
            vendor_id: 0x03C3,
            product_id: Some(0xFFFF),
        };
        assert!(!pattern.matches(&device));
    }

    #[test]
    fn display_format() {
        assert_eq!(VidPid::parse("03C3:120A").unwrap().to_string(), "03C3:120A");
        assert_eq!(VidPid::parse("03C3:*").unwrap().to_string(), "03C3:*");
    }

    #[test]
    fn parse_device_id_string() {
        let vp = parse_device_id_vidpid(r"USB\VID_03C3&PID_120A\6&1234").unwrap();
        assert_eq!(vp.vendor_id, 0x03C3);
        assert_eq!(vp.product_id, Some(0x120A));
    }

    #[test]
    fn parse_device_id_no_pid() {
        assert!(parse_device_id_vidpid(r"USB\VID_03C3\something").is_none());
    }
}
