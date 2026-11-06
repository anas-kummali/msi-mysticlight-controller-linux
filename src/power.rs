//! Hardware Power Supply and Laptop Battery Diagnostics.

use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerInfo {
    pub is_laptop: bool,
    pub ac_online: bool,
    pub battery_percent: Option<u8>,
    pub battery_status: String,
}

impl Default for PowerInfo {
    fn default() -> Self {
        Self {
            is_laptop: false,
            ac_online: true,
            battery_percent: None,
            battery_status: "AC Powered / Desktop".to_string(),
        }
    }
}

/// Query system AC adapter and battery status from `/sys/class/power_supply`.
pub fn query_power_info() -> PowerInfo {
    let base_path = Path::new("/sys/class/power_supply");
    if !base_path.exists() {
        return PowerInfo::default();
    }

    let entries = match fs::read_dir(base_path) {
        Ok(e) => e,
        Err(_) => return PowerInfo::default(),
    };

    let mut ac_online = false;
    let mut battery_found = false;
    let mut battery_percent = None;
    let mut battery_status = String::from("Unknown");

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        if name.starts_with("ADP") || name.starts_with("AC") {
            if let Ok(online_val) = fs::read_to_string(path.join("online")) {
                if online_val.trim() == "1" {
                    ac_online = true;
                }
            }
        } else if name.starts_with("BAT") {
            battery_found = true;
            if let Ok(cap_str) = fs::read_to_string(path.join("capacity")) {
                if let Ok(pct) = cap_str.trim().parse::<u8>() {
                    battery_percent = Some(pct);
                }
            }
            if let Ok(status_str) = fs::read_to_string(path.join("status")) {
                let st = status_str.trim();
                if !st.is_empty() {
                    battery_status = st.to_string();
                }
            }
        }
    }

    PowerInfo {
        is_laptop: battery_found,
        ac_online,
        battery_percent,
        battery_status: if battery_found {
            battery_status
        } else {
            "AC Powered / No Battery Detected".to_string()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_info_default() {
        let p = PowerInfo::default();
        assert!(!p.is_laptop);
        assert!(p.ac_online);
        assert_eq!(p.battery_percent, None);
    }

    #[test]
    fn test_query_power_info_non_panicking() {
        // Querying power info should never panic regardless of host environment
        let _ = query_power_info();
    }
}
