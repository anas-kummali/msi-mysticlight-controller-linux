//! RGB color representation, palette presets, and color parsing utilities.

use std::fmt;
use crate::error::ControllerError;

/// Named preset RGB colors supported by MSI MysticLight.
pub const COLORS: &[(&str, (u8, u8, u8))] = &[
    ("white", (255, 255, 255)),
    ("warmwhite", (255, 220, 180)),
    ("red", (255, 0, 0)),
    ("green", (0, 255, 0)),
    ("blue", (0, 120, 255)),
    ("cyan", (0, 255, 255)),
    ("magenta", (255, 0, 255)),
    ("purple", (160, 32, 240)),
    ("yellow", (255, 220, 0)),
    ("orange", (255, 100, 0)),
    ("ice", (180, 220, 255)),
];

/// An 8-bit RGB color value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// Create a new RGB color.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Match a named color from the standard palette (case-insensitive).
    pub fn from_name(name: &str) -> Option<Self> {
        let needle = name.trim().to_lowercase();
        COLORS
            .iter()
            .find(|(n, _)| *n == needle)
            .map(|(_, (r, g, b))| Self::new(*r, *g, *b))
    }

    /// Parse a 3-character (#RGB) or 6-character (#RRGGBB) hex color string.
    pub fn from_hex(val: &str) -> Result<Self, ControllerError> {
        let h = val.trim().trim_start_matches('#');
        let expanded: String;
        let hex_str = if h.len() == 3 {
            expanded = h.chars().flat_map(|c| [c, c]).collect();
            &expanded
        } else if h.len() == 6 {
            h
        } else {
            return Err(ControllerError::InvalidColor(format!(
                "Invalid hexadecimal color value: '{}'",
                val
            )));
        };

        let r = u8::from_str_radix(&hex_str[0..2], 16)
            .map_err(|_| ControllerError::InvalidColor(format!("Invalid hexadecimal color value: '{}'", val)))?;
        let g = u8::from_str_radix(&hex_str[2..4], 16)
            .map_err(|_| ControllerError::InvalidColor(format!("Invalid hexadecimal color value: '{}'", val)))?;
        let b = u8::from_str_radix(&hex_str[4..6], 16)
            .map_err(|_| ControllerError::InvalidColor(format!("Invalid hexadecimal color value: '{}'", val)))?;

        Ok(Self::new(r, g, b))
    }

    /// Parse a color string which can be either a preset name or a hex code.
    pub fn parse(val: &str) -> Result<Self, ControllerError> {
        if let Some(c) = Self::from_name(val) {
            return Ok(c);
        }
        Self::from_hex(val).map_err(|_| {
            let names: Vec<&str> = COLORS.iter().map(|(n, _)| *n).collect();
            ControllerError::InvalidColor(format!(
                "Invalid color: '{}'. Use hex (#RRGGBB) or named color ({})",
                val,
                names.join(", ")
            ))
        })
    }

    /// Scale color channels according to brightness level [0..10].
    pub fn scale(self, brightness: u8) -> Self {
        let factor = (brightness.min(10) as f32 / 10.0).clamp(0.0, 1.0);
        Self::new(
            (self.r as f32 * factor) as u8,
            (self.g as f32 * factor) as u8,
            (self.b as f32 * factor) as u8,
        )
    }

    /// Return as (r, g, b) tuple.
    pub const fn to_tuple(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::new(r, g, b)
    }
}

impl From<Rgb> for (u8, u8, u8) {
    fn from(c: Rgb) -> Self {
        c.to_tuple()
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Parse color name or hex (#RRGGBB) and scale by brightness level [0..10].
pub fn parse_color(val: &str, brightness: u8) -> Result<(u8, u8, u8), String> {
    Rgb::parse(val)
        .map(|c| c.scale(brightness).to_tuple())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("white", 10).unwrap(), (255, 255, 255));
        assert_eq!(parse_color("red", 10).unwrap(), (255, 0, 0));
        assert_eq!(parse_color("blue", 10).unwrap(), (0, 120, 255));
        assert_eq!(parse_color("cyan", 10).unwrap(), (0, 255, 255));
        assert_eq!(parse_color("WHITE", 10).unwrap(), (255, 255, 255));
        assert_eq!(parse_color("white", 5).unwrap(), (127, 127, 127));
        assert_eq!(parse_color("white", 0).unwrap(), (0, 0, 0));
    }

    #[test]
    fn test_parse_color_hex() {
        assert_eq!(parse_color("#FF0000", 10).unwrap(), (255, 0, 0));
        assert_eq!(parse_color("00ff00", 10).unwrap(), (0, 255, 0));
        assert_eq!(parse_color("#00f", 10).unwrap(), (0, 0, 255));
        assert_eq!(parse_color("#fff", 5).unwrap(), (127, 127, 127));
        assert!(parse_color("notacolor", 10).is_err());
        assert!(parse_color("#GGFFFF", 10).is_err());
    }

    #[test]
    fn test_rgb_struct() {
        let c = Rgb::new(100, 150, 200);
        assert_eq!(c.to_tuple(), (100, 150, 200));
        let scaled = c.scale(5);
        assert_eq!(scaled.r, 50);
        assert_eq!(scaled.g, 75);
        assert_eq!(scaled.b, 100);
        assert_eq!(format!("{}", c), "#6496C8");
    }
}

