//! Hardware constants, animation mode definitions, and known device identifiers.

pub use crate::color::COLORS;
pub use crate::matrix::ROW_ZONES;

/// Linux HID feature report ioctl request code for 64-byte payload.
/// _IOWR('H', 0x06, [u8; 64]) = 0xC0404806.
pub const HIDIOCSFEATURE_64: u64 = 0xC0404806;

/// Linux HID get feature report ioctl request code for 64-byte payload.
/// _IOWR('H', 0x07, [u8; 64]) = 0xC0404807.
pub const HIDIOCGFEATURE_64: u64 = 0xC0404807;

// Standard hardware animation modes
pub const MODE_OFF: u8 = 0;
pub const MODE_STATIC: u8 = 1;
pub const MODE_BREATHING: u8 = 2;
pub const MODE_CYCLE: u8 = 3;
pub const MODE_WAVE: u8 = 4;

// Wave animation directions
pub const WAVE_RTL: u8 = 0;
pub const WAVE_LTR: u8 = 1;

// Supported USB Vendor IDs
pub const VENDOR_MSI: &str = "1462";
pub const VENDOR_REALTEK_MSI: &str = "0DB0";

// Product IDs known in the MSI MysticLight family
pub const KNOWN_PIDS: &[(&str, &str)] = &[
    ("1462:1603", "MSI Vector 16 HX AI (MS-1603) - 24-Zone"),
    ("0DB0:1605", "MSI Vector A18 HX (MS-1605) - 24-Zone"),
    ("0DB0:1801", "MSI Crosshair 16 Max HX (MS-1801) - 4-Zone"),
    ("0DB0:1606", "MSI Cyborg 15 (MS-1606) - 4-Zone"),
    ("1462:1601", "MSI Katana 15 HX (MS-1565) - 4-Zone"),
    ("1462:1562", "MSI Delta 15 (MS-1562) - 4-Zone"),
    ("1462:1563", "MSI Alpha 15/17 (MS-1563) - 4-Zone"),
    ("1462:1564", "MSI Stealth / Raider (MS-1564) - 4-Zone"),
];
