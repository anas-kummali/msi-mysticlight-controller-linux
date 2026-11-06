//! Sysfs device discovery and hardware status inspection.

use std::fs::OpenOptions;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};

use crate::constants::{KNOWN_PIDS, VENDOR_MSI, VENDOR_REALTEK_MSI};
use crate::matrix::{TOTAL_ROWS, TOTAL_ZONES};

/// Device status and diagnostic information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub node: Option<String>,
    pub present: bool,
    pub accessible: bool,
    pub total_zones: usize,
    pub total_rows: usize,
}

impl DeviceInfo {
    /// Probe device status and permissions for the specified node.
    pub fn query(device_node: Option<&Path>) -> Self {
        match device_node {
            Some(node) => {
                let present = node.exists();
                let accessible = if present {
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(node)
                        .is_ok()
                } else {
                    false
                };
                Self {
                    node: Some(node.display().to_string()),
                    present,
                    accessible,
                    total_zones: TOTAL_ZONES,
                    total_rows: TOTAL_ROWS,
                }
            }
            None => Self {
                node: None,
                present: false,
                accessible: false,
                total_zones: TOTAL_ZONES,
                total_rows: TOTAL_ROWS,
            },
        }
    }
}

/// Verify that a path is a legitimate character device under /dev/hidraw.
fn validate_hidraw_path(path: &Path) -> Option<PathBuf> {
    let resolved = std::fs::canonicalize(path).ok()?;
    let meta = std::fs::metadata(&resolved).ok()?;
    let is_chr = meta.file_type().is_char_device();
    let starts_dev = resolved.starts_with("/dev");
    let is_hidraw = resolved
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.starts_with("hidraw"))
        .unwrap_or(false);

    if is_chr && starts_dev && is_hidraw {
        Some(resolved)
    } else {
        None
    }
}

/// Scan `/sys/class/hidraw/` for MSI MysticLight USB hardware nodes.
pub fn find_device() -> Option<PathBuf> {
    // 1. Honor explicit environment variable override if valid and safe
    if let Ok(override_path) = std::env::var("MSI_KEYBOARD_DEVICE") {
        if let Some(valid_node) = validate_hidraw_path(Path::new(&override_path)) {
            return Some(valid_node);
        }
    }

    // 2. Discover via sysfs uevent inspection
    let mut candidates: Vec<(PathBuf, String, String)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/sys/class/hidraw") {
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
        paths.sort();

        for path in paths {
            let uevent_path = path.join("device").join("uevent");
            let content = match std::fs::read_to_string(&uevent_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let up = content.to_uppercase();
            let mut ident = String::new();
            let mut name = String::new();

            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("HID_ID=") {
                    let bits: Vec<&str> = rest.split(':').collect();
                    if bits.len() == 3 {
                        let v = if bits[1].len() >= 4 {
                            &bits[1][bits[1].len() - 4..]
                        } else {
                            bits[1]
                        };
                        let p = if bits[2].len() >= 4 {
                            &bits[2][bits[2].len() - 4..]
                        } else {
                            bits[2]
                        };
                        ident = format!("{}:{}", v.to_uppercase(), p.to_uppercase());
                    }
                } else if let Some(rest) = line.strip_prefix("HID_NAME=") {
                    name = rest.trim().to_string();
                }
            }

            let has_vendor = up.contains(VENDOR_MSI) || up.contains(VENDOR_REALTEK_MSI);
            let is_known = KNOWN_PIDS.iter().any(|(p, _)| *p == ident);
            let is_mystic = up.replace(' ', "").contains("MYSTICLIGHT") || is_known;

            if is_mystic && has_vendor {
                if let Some(file_name) = path.file_name() {
                    let node = PathBuf::from("/dev").join(file_name);
                    if node.exists() {
                        candidates.push((node, ident, name));
                    }
                }
            }
        }
    }

    if candidates.is_empty() {
        let fallback = PathBuf::from("/dev/hidraw3");
        if fallback.exists() {
            if let Ok(meta) = std::fs::metadata(&fallback) {
                if meta.file_type().is_char_device() {
                    return Some(fallback);
                }
            }
        }
        return None;
    }

    // Prefer exact MS-1603 hardware PID if present
    candidates.sort_by(|a, b| {
        let a_score = (a.1 != "1462:1603", !KNOWN_PIDS.iter().any(|(p, _)| *p == a.1));
        let b_score = (b.1 != "1462:1603", !KNOWN_PIDS.iter().any(|(p, _)| *p == b.1));
        a_score.cmp(&b_score)
    });

    Some(candidates[0].0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_device_override_rejection() {
        // Non-existent or non-char file should never be accepted
        unsafe {
            std::env::set_var("MSI_KEYBOARD_DEVICE", "/etc/passwd");
        }
        let dev = find_device();
        if let Some(ref d) = dev {
            assert_ne!(d.to_str().unwrap(), "/etc/passwd");
            assert!(d.to_str().unwrap().starts_with("/dev/hidraw"));
        }
        unsafe {
            std::env::remove_var("MSI_KEYBOARD_DEVICE");
        }
    }

    #[test]
    fn test_device_info_query_none() {
        let info = DeviceInfo::query(None);
        assert_eq!(info.node, None);
        assert!(!info.present);
        assert!(!info.accessible);
        assert_eq!(info.total_zones, 24);
        assert_eq!(info.total_rows, 6);
    }
}
