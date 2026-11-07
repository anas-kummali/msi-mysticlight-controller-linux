//! MSI Mystic Light Controller for Linux (Library)
//!
//! High-performance controller and GUI for MSI MysticLight
//! 24-Zone and 4-Zone keyboard backlights on Linux.

pub mod bridge;
pub mod cli;
pub mod color;
pub mod constants;
pub mod controller;
pub mod device;
pub mod error;
pub mod gradient;
pub mod matrix;
pub mod power;
pub mod profile;
pub mod protocol;
pub mod theme;

#[cfg(feature = "gui")]
pub mod gui;

pub const VERSION: &str = "1.2.0";

// Re-export primary public types and utilities
pub use color::{parse_color, Rgb, COLORS};
pub use constants::*;
pub use controller::MSIKeyboardController;
pub use device::{find_device, DeviceInfo};
pub use error::{ControllerError, ControllerResult};
pub use gradient::{generate_gradient, lerp_rgb, GradientDirection};
pub use matrix::{get_row_zones, is_valid_row, is_valid_zone, row_for_zone, ROW_ZONES};
pub use power::{query_power_info, PowerInfo};
pub use profile::{load_custom_profiles, save_custom_profiles, CustomProfile};
pub use protocol::*;
pub use theme::{get_builtin_themes, Theme};
