//! Strongly-typed error hierarchy for the MSI MysticLight controller.

use std::fmt;

/// Result alias for controller operations.
pub type ControllerResult<T> = Result<T, ControllerError>;

/// Represents errors that can occur during device discovery, HID communication,
/// or user configuration.
#[derive(Debug)]
pub enum ControllerError {
    /// Standard I/O or kernel communication error.
    Io(std::io::Error),
    /// No supported MSI MysticLight HID device was found on the system.
    DeviceNotFound,
    /// Permission denied opening the hidraw device node.
    PermissionDenied(String),
    /// The specified zone index is outside the valid range [0..23].
    InvalidZone { zone: u8, max: u8 },
    /// The specified row index is outside the valid range [1..6].
    InvalidRow { row: u8, max: u8 },
    /// The specified brightness level is outside the valid range [0..10].
    InvalidBrightness { brightness: u8, max: u8 },
    /// The color string could not be parsed as a hex code or known palette name.
    InvalidColor(String),
    /// Low-level protocol or packet generation failure.
    ProtocolError(String),
}

impl fmt::Display for ControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {}", err),
            Self::DeviceNotFound => {
                write!(f, "MSI MysticLight HID device not found in /sys/class/hidraw/.")
            }
            Self::PermissionDenied(msg) => write!(f, "{}", msg),
            Self::InvalidZone { zone, max } => {
                write!(f, "Zone {} out of range [0..{}].", zone, max)
            }
            Self::InvalidRow { row, max } => {
                write!(f, "Row {} out of range [1..{}].", row, max)
            }
            Self::InvalidBrightness { brightness, max } => {
                write!(f, "Brightness {} out of range [0..{}].", brightness, max)
            }
            Self::InvalidColor(msg) => write!(f, "{}", msg),
            Self::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl std::error::Error for ControllerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ControllerError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<String> for ControllerError {
    fn from(msg: String) -> Self {
        Self::ProtocolError(msg)
    }
}
