//! Mystic Light Keyboard Theme Management and Presets.
//!
//! Integrates OpenRGB profile concepts and LightShow theme palettes
//! into rich 24-zone hardware configurations.

use crate::color::Rgb;
use crate::controller::MSIKeyboardController;
use crate::matrix::{ROW_ZONES, TOTAL_ZONES};

/// A complete keyboard lighting theme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub primary_color: Rgb,
    pub zones: [Rgb; TOTAL_ZONES],
}

impl Theme {
    /// Create a theme with a uniform color across all 24 zones.
    pub fn solid(id: &'static str, name: &'static str, desc: &'static str, color: Rgb) -> Self {
        Self {
            id,
            name,
            description: desc,
            primary_color: color,
            zones: [color; TOTAL_ZONES],
        }
    }

    /// Create a theme with 6 distinct colors mapped to the 6 physical rows.
    pub fn rows(
        id: &'static str,
        name: &'static str,
        desc: &'static str,
        row_colors: [Rgb; 6],
    ) -> Self {
        let mut zones = [row_colors[0]; TOTAL_ZONES];
        for (r_idx, (_row_num, row_z)) in ROW_ZONES.iter().enumerate() {
            let color = row_colors[r_idx.min(5)];
            for &z in row_z {
                if (z as usize) < TOTAL_ZONES {
                    zones[z as usize] = color;
                }
            }
        }
        Self {
            id,
            name,
            description: desc,
            primary_color: row_colors[0],
            zones,
        }
    }

    /// Apply this theme to the physical keyboard.
    pub fn apply(
        &self,
        ctrl: &MSIKeyboardController,
        brightness: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // If all zones have identical color, use fast full-matrix set_color
        let first = self.zones[0];
        if self.zones.iter().all(|&c| c == first) {
            ctrl.set_color(first.r, first.g, first.b, brightness, None, None)?;
            return Ok(());
        }

        // Apply per-zone lighting map
        for (z, &rgb) in self.zones.iter().enumerate() {
            let scaled = rgb.scale(brightness);
            ctrl.set_color(
                scaled.r,
                scaled.g,
                scaled.b,
                10, // scaled is already adjusted
                None,
                Some(z as u8),
            )?;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        Ok(())
    }
}

/// Catalog of pre-configured themes.
pub fn get_builtin_themes() -> Vec<Theme> {
    vec![
        Theme::solid(
            "sovereign-white",
            "Sovereign White",
            "Clean 6500K workstation illumination across all 6 rows",
            Rgb::new(255, 255, 255),
        ),
        Theme::rows(
            "tokyo-night",
            "Tokyo Night",
            "Iconic dark palette: Blue, Purple, Cyan, Green, Orange, Red",
            [
                Rgb::new(122, 162, 247), // Row 1: Tokyo Blue
                Rgb::new(187, 154, 247), // Row 2: Tokyo Purple
                Rgb::new(125, 207, 255), // Row 3: Tokyo Cyan
                Rgb::new(158, 206, 106), // Row 4: Tokyo Green
                Rgb::new(224, 175, 104), // Row 5: Tokyo Orange
                Rgb::new(247, 118, 142), // Row 6: Tokyo Red/Pink
            ],
        ),
        Theme::rows(
            "cyberpunk-2077",
            "Cyberpunk Neon",
            "High-voltage Night City: Neon Yellow, Hot Pink, and Cyan",
            [
                Rgb::new(252, 238, 10), // Row 1: Neon Yellow
                Rgb::new(0, 240, 255),   // Row 2: Cyan
                Rgb::new(255, 0, 60),    // Row 3: Hot Red/Pink
                Rgb::new(252, 238, 10), // Row 4: Neon Yellow
                Rgb::new(0, 240, 255),   // Row 5: Cyan
                Rgb::new(255, 0, 60),    // Row 6: Hot Red/Pink
            ],
        ),
        Theme::rows(
            "synthwave",
            "Synthwave / Sunset",
            "80s retro sunset: Deep Magenta, Royal Violet, and Hot Orange",
            [
                Rgb::new(255, 20, 147), // Row 1: Deep Pink
                Rgb::new(138, 43, 226), // Row 2: Blue Violet
                Rgb::new(255, 69, 0),   // Row 3: Red Orange
                Rgb::new(255, 215, 0),  // Row 4: Gold
                Rgb::new(255, 20, 147), // Row 5: Deep Pink
                Rgb::new(0, 255, 255),  // Row 6: Neon Cyan
            ],
        ),
        Theme::rows(
            "matrix-code",
            "Matrix Code",
            "Phosphor digital rain cascading down through dark green tiers",
            [
                Rgb::new(180, 255, 180), // Row 1: Bright Flash
                Rgb::new(0, 255, 65),    // Row 2: Phosphor Green
                Rgb::new(0, 200, 45),    // Row 3: Medium Matrix
                Rgb::new(0, 143, 17),    // Row 4: Deep Terminal
                Rgb::new(0, 90, 10),     // Row 5: Dark Shadow
                Rgb::new(0, 59, 0),      // Row 6: Deep Bass
            ],
        ),
        Theme::rows(
            "arctic-frost",
            "Arctic Frost",
            "Glacial serenity with sub-zero cyan, ice blue, and pure snow",
            [
                Rgb::new(255, 255, 255), // Row 1: Snow
                Rgb::new(180, 220, 255), // Row 2: Ice Blue
                Rgb::new(0, 255, 255),   // Row 3: Electric Cyan
                Rgb::new(100, 200, 255), // Row 4: Frost Blue
                Rgb::new(180, 220, 255), // Row 5: Ice Blue
                Rgb::new(255, 255, 255), // Row 6: Snow
            ],
        ),
        Theme::rows(
            "nord-glacier",
            "Nord Aurora",
            "Authentic Arctic Nord palette: Frost blues and Aurora green",
            [
                Rgb::new(136, 192, 208), // Row 1: Nord Frost 8
                Rgb::new(129, 161, 193), // Row 2: Nord Frost 9
                Rgb::new(94, 129, 172),  // Row 3: Nord Frost 10
                Rgb::new(163, 190, 140), // Row 4: Nord Aurora Green
                Rgb::new(235, 203, 139), // Row 5: Nord Aurora Yellow
                Rgb::new(191, 97, 106),  // Row 6: Nord Aurora Red
            ],
        ),
        Theme::solid(
            "blood-red",
            "Tactical Crimson",
            "High-contrast battle crimson designed for dark environments",
            Rgb::new(255, 0, 0),
        ),
        Theme::solid(
            "warm-amber",
            "Warm Amber 2700K",
            "Relaxing incandescent eye-care illumination",
            Rgb::new(255, 180, 80),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_themes_count() {
        let themes = get_builtin_themes();
        assert!(themes.len() >= 8);
        for t in &themes {
            assert_eq!(t.zones.len(), 24);
        }
    }

    #[test]
    fn test_theme_row_assignment() {
        let t = Theme::rows(
            "test",
            "Test",
            "Desc",
            [
                Rgb::new(1, 1, 1),
                Rgb::new(2, 2, 2),
                Rgb::new(3, 3, 3),
                Rgb::new(4, 4, 4),
                Rgb::new(5, 5, 5),
                Rgb::new(6, 6, 6),
            ],
        );
        // Row 1 zones 0..3 should have (1, 1, 1)
        assert_eq!(t.zones[0], Rgb::new(1, 1, 1));
        assert_eq!(t.zones[3], Rgb::new(1, 1, 1));
        // Row 6 zones 20..23 should have (6, 6, 6)
        assert_eq!(t.zones[20], Rgb::new(6, 6, 6));
        assert_eq!(t.zones[23], Rgb::new(6, 6, 6));
    }
}
