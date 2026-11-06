//! High-level hardware controller interface for MSI 24-Zone & 4-Zone Keyboards.

use std::fs::OpenOptions;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};

use crate::constants::{
    COLORS, HIDIOCSFEATURE_64, KNOWN_PIDS, MODE_OFF, MODE_STATIC,
    ROW_ZONES, VENDOR_MSI, VENDOR_REALTEK_MSI, WAVE_LTR,
};
use crate::protocol::{
    build_all_zones_prep_packet, build_effect_packet, build_format_a_packet,
    build_format_b_static_packet, build_msi_center_color_packet, build_off_packet,
    build_zone_mask_packet,
};

extern "C" {
    fn ioctl(fd: std::os::raw::c_int, request: std::os::raw::c_ulong, ...) -> std::os::raw::c_int;
}

/// Device status information report.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub node: Option<String>,
    pub present: bool,
    pub accessible: bool,
    pub total_zones: usize,
    pub total_rows: usize,
}

/// Scan /sys/class/hidraw/ for MSI MysticLight USB hardware nodes.
pub fn find_device() -> Option<PathBuf> {
    // Check env override first with strict path and device type validation
    if let Ok(override_path) = std::env::var("MSI_KEYBOARD_DEVICE") {
        let p = Path::new(&override_path);
        if let Ok(resolved) = std::fs::canonicalize(p) {
            if let Ok(meta) = std::fs::metadata(&resolved) {
                let is_chr = meta.file_type().is_char_device();
                let starts_dev = resolved.starts_with("/dev");
                let is_hidraw = resolved
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.starts_with("hidraw"))
                    .unwrap_or(false);
                if is_chr && starts_dev && is_hidraw {
                    return Some(resolved);
                }
            }
        }
    }

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

    candidates.sort_by(|a, b| {
        let a_score = (a.1 != "1462:1603", !KNOWN_PIDS.iter().any(|(p, _)| *p == a.1));
        let b_score = (b.1 != "1462:1603", !KNOWN_PIDS.iter().any(|(p, _)| *p == b.1));
        a_score.cmp(&b_score)
    });

    Some(candidates[0].0.clone())
}

/// Parse color name or hex (#RRGGBB) and scale by brightness level [0..10].
pub fn parse_color(val: &str, brightness: u8) -> Result<(u8, u8, u8), String> {
    let trimmed = val.trim().to_lowercase();
    let (r, g, b) = if let Some(&(_, rgb)) = COLORS.iter().find(|(name, _)| *name == trimmed) {
        rgb
    } else {
        let h = trimmed.trim_start_matches('#');
        let expanded: String;
        let hex_str = if h.len() == 3 {
            expanded = h.chars().flat_map(|c| [c, c]).collect();
            &expanded
        } else if h.len() == 6 {
            h
        } else {
            let names: Vec<&str> = COLORS.iter().map(|(n, _)| *n).collect();
            return Err(format!(
                "Invalid color: '{}'. Use hex (#RRGGBB) or named color ({})",
                val,
                names.join(", ")
            ));
        };

        let r = u8::from_str_radix(&hex_str[0..2], 16)
            .map_err(|_| format!("Invalid hexadecimal color value: '{}'", val))?;
        let g = u8::from_str_radix(&hex_str[2..4], 16)
            .map_err(|_| format!("Invalid hexadecimal color value: '{}'", val))?;
        let b = u8::from_str_radix(&hex_str[4..6], 16)
            .map_err(|_| format!("Invalid hexadecimal color value: '{}'", val))?;
        (r, g, b)
    };

    let scale = (brightness.min(10) as f32 / 10.0).clamp(0.0, 1.0);
    Ok((
        (r as f32 * scale) as u8,
        (g as f32 * scale) as u8,
        (b as f32 * scale) as u8,
    ))
}

/// Manages raw HID communication with the MSI Keyboard backlight controller.
pub struct MSIKeyboardController {
    pub device_node: Option<PathBuf>,
    file: Option<std::fs::File>,
}

impl MSIKeyboardController {
    /// Create a new controller detecting device automatically.
    pub fn new() -> Self {
        Self {
            device_node: find_device(),
            file: None,
        }
    }

    /// Create a new controller targeting an explicit device path.
    pub fn with_device(path: impl Into<PathBuf>) -> Self {
        Self {
            device_node: Some(path.into()),
            file: None,
        }
    }

    /// Open read/write handle to the HID device node.
    pub fn open(&mut self) -> Result<(), std::io::Error> {
        let node = match &self.device_node {
            Some(n) => n.clone(),
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "MSI MysticLight HID device not found in /sys/class/hidraw/.",
                ));
            }
        };

        match OpenOptions::new().read(true).write(true).open(&node) {
            Ok(f) => {
                self.file = Some(f);
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!(
                        "Permission denied on {}. Ensure udev rule /etc/udev/rules.d/99-msi-mysticlight.rules is installed.",
                        node.display()
                    ),
                ))
            }
            Err(e) => Err(e),
        }
    }

    /// Close the open device file descriptor.
    pub fn close(&mut self) {
        self.file = None;
    }

    /// Send a 64-byte USB HID feature report using POSIX ioctl.
    pub fn send_feature(&self, data: &[u8]) -> Result<(), std::io::Error> {
        let file = self.file.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "Device handle is not open.")
        })?;

        let mut payload = [0u8; 64];
        let copy_len = data.len().min(64);
        payload[..copy_len].copy_from_slice(&data[..copy_len]);

        let fd: RawFd = file.as_raw_fd();
        let ret = unsafe {
            ioctl(
                fd,
                HIDIOCSFEATURE_64 as std::os::raw::c_ulong,
                payload.as_mut_ptr() as *mut std::ffi::c_void,
            )
        };

        if ret < 0 {
            let err = std::io::Error::last_os_error();
            let node_str = self
                .device_node
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".into());
            return Err(std::io::Error::new(
                err.kind(),
                format!(
                    "Failed sending USB HID feature report to {}: {}",
                    node_str, err
                ),
            ));
        }

        Ok(())
    }

    /// Set static color across all 6 rows (all 24 zones), a specific row, or a specific zone.
    pub fn set_color(
        &self,
        r: u8,
        g: u8,
        b: u8,
        brightness: u8,
        row: Option<u8>,
        zone: Option<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scale = (brightness.min(10) as f32 / 10.0).clamp(0.0, 1.0);
        let sr = (r as f32 * scale) as u8;
        let sg = (g as f32 * scale) as u8;
        let sb = (b as f32 * scale) as u8;
        let b_pct = (brightness.min(10) * 10).min(100);

        let p_b = build_format_b_static_packet(sr, sg, sb, 100, 0xFF);
        let p_c = build_msi_center_color_packet(sr, sg, sb, b_pct, 0x58, 0x32, 0x08);
        let p_a = build_format_a_packet(MODE_STATIC, 0, brightness, sr, sg, sb);

        if let Some(z) = zone {
            if z >= 24 {
                return Err(format!("Invalid zone: {}. Must be between 0 and 23.", z).into());
            }
            self.send_feature(&build_zone_mask_packet(z)?)?;
            self.send_feature(&p_b)?;
            self.send_feature(&p_c)?;
        } else if let Some(r_num) = row {
            let row_entry = ROW_ZONES
                .iter()
                .find(|(r, _)| *r == r_num)
                .ok_or_else(|| format!("Invalid row: {}. Supported rows: 1..6", r_num))?;

            for &z in &row_entry.1 {
                self.send_feature(&build_zone_mask_packet(z)?)?;
                self.send_feature(&p_b)?;
                self.send_feature(&p_c)?;
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        } else {
            // Full 24-Zone, 6-Row simultaneous activation
            self.send_feature(&build_all_zones_prep_packet())?;
            std::thread::sleep(std::time::Duration::from_millis(5));
            self.send_feature(&p_b)?;
            std::thread::sleep(std::time::Duration::from_millis(5));
            self.send_feature(&p_c)?;
            std::thread::sleep(std::time::Duration::from_millis(5));
            self.send_feature(&p_a)?;
            std::thread::sleep(std::time::Duration::from_millis(5));

            // Ensure every physical LED register is updated
            for z in 0..24 {
                self.send_feature(&build_zone_mask_packet(z)?)?;
                self.send_feature(&p_b)?;
                self.send_feature(&p_c)?;
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        }

        Ok(())
    }

    /// Completely power off keyboard LEDs across all 6 rows and all 24 zones.
    pub fn turn_off(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_feature(&build_all_zones_prep_packet())?;
        let p_b = build_off_packet();
        let p_a = build_format_a_packet(MODE_OFF, 0, 0, 0, 0, 0);

        self.send_feature(&p_b)?;
        self.send_feature(&p_a)?;

        for z in 0..24 {
            self.send_feature(&build_zone_mask_packet(z)?)?;
            self.send_feature(&p_b)?;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        Ok(())
    }

    /// Apply hardware-interpolated effects across all 24 zones and all 6 rows.
    pub fn set_effect(
        &self,
        mode: u8,
        colors: &[(u8, u8, u8)],
        speed: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.send_feature(&build_all_zones_prep_packet())?;
        std::thread::sleep(std::time::Duration::from_millis(10));

        let p = build_effect_packet(mode, colors, speed, WAVE_LTR);
        self.send_feature(&p)?;
        std::thread::sleep(std::time::Duration::from_millis(10));

        for z in 0..24 {
            self.send_feature(&build_zone_mask_packet(z)?)?;
            self.send_feature(&p)?;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        Ok(())
    }

    /// Query hardware availability and access permissions.
    pub fn get_device_info(&self) -> DeviceInfo {
        match &self.device_node {
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
                DeviceInfo {
                    node: Some(node.display().to_string()),
                    present,
                    accessible,
                    total_zones: 24,
                    total_rows: 6,
                }
            }
            None => DeviceInfo {
                node: None,
                present: false,
                accessible: false,
                total_zones: 24,
                total_rows: 6,
            },
        }
    }
}

impl Default for MSIKeyboardController {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MSIKeyboardController {
    fn drop(&mut self) {
        self.close();
    }
}
