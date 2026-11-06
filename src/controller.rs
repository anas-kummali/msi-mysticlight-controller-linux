//! Hardware controller interface for MSI 24-Zone & 4-Zone Keyboards.

use std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;

pub use crate::color::parse_color;
pub use crate::device::{find_device, DeviceInfo};
use crate::constants::{HIDIOCSFEATURE_64, MODE_OFF, MODE_STATIC, WAVE_LTR};
use crate::matrix::ROW_ZONES;
use crate::protocol::{
    build_all_zones_prep_packet, build_effect_packet, build_format_a_packet,
    build_format_b_static_packet, build_msi_center_color_packet, build_off_packet,
    build_zone_mask_packet,
};

extern "C" {
    fn ioctl(fd: std::os::raw::c_int, request: std::os::raw::c_ulong, ...) -> std::os::raw::c_int;
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
        DeviceInfo::query(self.device_node.as_deref())
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
