//! MSI Mystic Light Controller for Linux (Library)
//!
//! High-performance, zero-external-dependency controller for MSI MysticLight
//! 24-Zone and 4-Zone keyboard backlights on Linux.

pub mod cli;
pub mod color;
pub mod constants;
pub mod controller;
pub mod device;
pub mod error;
pub mod matrix;
pub mod protocol;

pub const VERSION: &str = "1.0.0";

// Re-export primary public types and utilities
pub use color::{parse_color, Rgb, COLORS};
pub use constants::*;
pub use controller::MSIKeyboardController;
pub use device::{find_device, DeviceInfo};
pub use error::{ControllerError, ControllerResult};
pub use matrix::{get_row_zones, is_valid_row, is_valid_zone, row_for_zone, ROW_ZONES};
pub use protocol::*;
