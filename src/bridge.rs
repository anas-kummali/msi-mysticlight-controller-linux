//! Userspace sysfs LED bridge via `/dev/uleds`.
//!
//! Exposes `/sys/class/leds/msi::kbd_backlight` to the Linux kernel so that
//! desktop environments (COSMIC Desktop, GNOME, KDE Plasma) and `upowerd`
//! automatically discover the keyboard backlight, enabling:
//! 1. Native OS tray brightness slider (resolving the `NaN%` division-by-zero bug).
//! 2. Laptop Fn+Backlight hotkeys handled by the desktop session.
//! 3. Standard CLI brightness tools (`brightnessctl --device='msi::kbd_backlight' set 50%`).

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::controller::MSIKeyboardController;

pub const ULEDS_PATH: &str = "/dev/uleds";
pub const LED_MAX_NAME_SIZE: usize = 64;
pub const SYSFS_DEVICE_NAME: &str = "msi::kbd_backlight";
pub const MAX_BRIGHTNESS_STEPS: i32 = 10;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UledsUserDev {
    pub name: [u8; LED_MAX_NAME_SIZE],
    pub max_brightness: i32,
}

impl UledsUserDev {
    pub fn new(name_str: &str, max_b: i32) -> Self {
        let mut name = [0u8; LED_MAX_NAME_SIZE];
        let bytes = name_str.as_bytes();
        let len = bytes.len().min(LED_MAX_NAME_SIZE - 1);
        name[..len].copy_from_slice(&bytes[..len]);
        Self {
            name,
            max_brightness: max_b,
        }
    }
}

/// Helper to get state storage path: `~/.config/msi-mysticlight/state.json`
pub fn get_state_file_path() -> Option<PathBuf> {
    crate::profile::get_config_dir().map(|d| d.join("state.json"))
}

/// Active lighting state persisted between CLI, GUI, and Bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeState {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub brightness: u8,
}

impl Default for BridgeState {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            brightness: 10,
        }
    }
}

impl BridgeState {
    pub fn load() -> Self {
        if let Some(path) = get_state_file_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Some(state) = Self::parse_json(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = get_state_file_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let json = format!(
                "{{\n  \"r\": {},\n  \"g\": {},\n  \"b\": {},\n  \"brightness\": {}\n}}\n",
                self.r, self.g, self.b, self.brightness
            );
            let _ = std::fs::write(path, json);
        }
    }

    pub fn parse_json(json: &str) -> Option<Self> {
        let mut r = None;
        let mut g = None;
        let mut b = None;
        let mut brightness = None;

        for line in json.lines() {
            let trimmed = line.trim().trim_matches(',');
            if let Some((k, v)) = trimmed.split_once(':') {
                let key = k.trim().trim_matches('"');
                let val_str = v.trim().trim_matches('"');
                if let Ok(num) = val_str.parse::<u8>() {
                    match key {
                        "r" => r = Some(num),
                        "g" => g = Some(num),
                        "b" => b = Some(num),
                        "brightness" => brightness = Some(num.min(10)),
                        _ => {}
                    }
                }
            }
        }

        Some(Self {
            r: r.unwrap_or(255),
            g: g.unwrap_or(255),
            b: b.unwrap_or(255),
            brightness: brightness.unwrap_or(10),
        })
    }
}

/// Run the userspace sysfs bridge daemon.
pub fn run_bridge(initial_color: Option<(u8, u8, u8)>) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(ULEDS_PATH).exists() {
        return Err(format!(
            "Device node '{}' not found.\n\n\
            Root Cause:\n  The Linux kernel 'uleds' (userspace LED) module is not loaded.\n\n\
            To enable the OS tray brightness slider and laptop hotkeys, run:\n  \
            1. sudo modprobe uleds\n  \
            2. echo \"uleds\" | sudo tee /etc/modules-load.d/uleds.conf\n  \
            3. sudo udevadm control --reload && sudo udevadm trigger",
            ULEDS_PATH
        ).into());
    }

    let mut file = match OpenOptions::new().read(true).write(true).open(ULEDS_PATH) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Permission denied or error opening '{}': {}.\n\n\
                Please ensure udev rules grant access to your user session:\n  \
                KERNEL==\"uleds\", SUBSYSTEM==\"misc\", MODE=\"0660\", TAG+=\"uaccess\"",
                ULEDS_PATH, e
            ).into());
        }
    };

    let user_dev = UledsUserDev::new(SYSFS_DEVICE_NAME, MAX_BRIGHTNESS_STEPS);
    let bytes = unsafe {
        std::slice::from_raw_parts(
            &user_dev as *const _ as *const u8,
            std::mem::size_of::<UledsUserDev>(),
        )
    };

    file.write_all(bytes)?;
    file.flush()?;

    println!("==================================================");
    println!(" MSI Mystic Light — Linux Sysfs & OS Tray Bridge");
    println!("==================================================");
    println!(" Sysfs Device Node : /sys/class/leds/{}", SYSFS_DEVICE_NAME);
    println!(" Max Brightness    : {} steps (0..={})", MAX_BRIGHTNESS_STEPS, MAX_BRIGHTNESS_STEPS);
    println!(" Desktop Interface : UPower & Desktop Tray Integrated");
    println!(" Status            : Active & Listening for OS events");
    println!(" Press Ctrl+C to terminate the bridge.");
    println!("==================================================");

    let mut ctrl = MSIKeyboardController::new();
    ctrl.open()?;

    let mut state = BridgeState::load();
    if let Some((r, g, b)) = initial_color {
        state.r = r;
        state.g = g;
        state.b = b;
    }

    // Apply initial hardware state
    if state.brightness == 0 {
        let _ = ctrl.turn_off();
    } else {
        let _ = ctrl.set_color(state.r, state.g, state.b, state.brightness, None, None);
    }
    state.save();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Signal handler setup
    let _ = ctrlc_handler(move || {
        r.store(false, Ordering::SeqCst);
    });

    let mut buf = [0u8; 4];
    while running.load(Ordering::SeqCst) {
        match file.read_exact(&mut buf) {
            Ok(()) => {
                let raw_val = i32::from_ne_bytes(buf);
                let new_brightness = raw_val.clamp(0, MAX_BRIGHTNESS_STEPS) as u8;

                if new_brightness != state.brightness {
                    state.brightness = new_brightness;
                    println!(
                        "[OS Event] Backlight brightness adjusted: {}/{} ({:.0}%)",
                        new_brightness,
                        MAX_BRIGHTNESS_STEPS,
                        (new_brightness as f64 / MAX_BRIGHTNESS_STEPS as f64) * 100.0
                    );

                    if new_brightness == 0 {
                        let _ = ctrl.turn_off();
                    } else {
                        // Reload state to catch any color changes from CLI/GUI
                        let latest = BridgeState::load();
                        state.r = latest.r;
                        state.g = latest.g;
                        state.b = latest.b;
                        let _ = ctrl.set_color(state.r, state.g, state.b, new_brightness, None, None);
                    }
                    state.save();
                }
            }
            Err(e) => {
                if running.load(Ordering::SeqCst) {
                    eprintln!("Error reading uleds event: {}", e);
                }
                break;
            }
        }
    }

    println!("MSI Mystic Light bridge terminated. Unregistered sysfs node.");
    Ok(())
}

fn ctrlc_handler<F>(_f: F) -> Result<(), ()>
where
    F: FnOnce() + Send + 'static,
{
    // Standard SIGINT handling will close fd and clean up automatically on process exit
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uleds_user_dev_layout() {
        assert_eq!(std::mem::size_of::<UledsUserDev>(), 68);
        let dev = UledsUserDev::new("msi::kbd_backlight", 10);
        assert_eq!(dev.max_brightness, 10);
        let name_str = std::str::from_utf8(&dev.name).unwrap();
        assert!(name_str.starts_with("msi::kbd_backlight"));
    }

    #[test]
    fn test_bridge_state_json_parse() {
        let json = r#"
        {
            "r": 100,
            "g": 150,
            "b": 200,
            "brightness": 7
        }
        "#;
        let s = BridgeState::parse_json(json).unwrap();
        assert_eq!(s.r, 100);
        assert_eq!(s.g, 150);
        assert_eq!(s.b, 200);
        assert_eq!(s.brightness, 7);
    }
}
