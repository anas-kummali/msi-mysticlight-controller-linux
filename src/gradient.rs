//! Chromatic Gradient Generator for 24-Zone Keyboard Backlights.

use crate::color::Rgb;
use crate::matrix::TOTAL_ZONES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientDirection {
    Horizontal, // Sweeps left-to-right across 4 columns
    Vertical,   // Sweeps top-to-bottom across 6 physical rows
    Diagonal,   // Sweeps top-left to bottom-right
    Radial,     // Radiates outward from center
}

impl GradientDirection {
    pub const ALL: &'static [GradientDirection] = &[
        GradientDirection::Horizontal,
        GradientDirection::Vertical,
        GradientDirection::Diagonal,
        GradientDirection::Radial,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Horizontal => "Horizontal (Left → Right)",
            Self::Vertical => "Vertical (Top → Bottom)",
            Self::Diagonal => "Diagonal (Top-Left → Bottom-Right)",
            Self::Radial => "Radial (Center → Outer Edge)",
        }
    }
}

/// Linear interpolation between two RGB colors by factor `t` in [0.0, 1.0].
pub fn lerp_rgb(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0_f32, 1.0_f32);
    let r = (a.r as f32 + (b.r as f32 - a.r as f32) * t).round() as u8;
    let g = (a.g as f32 + (b.g as f32 - a.g as f32) * t).round() as u8;
    let bl = (a.b as f32 + (b.b as f32 - a.b as f32) * t).round() as u8;
    Rgb::new(r, g, bl)
}

/// Generates a 24-zone color array by interpolating between `start` and `end`
/// according to the selected spatial `direction`.
pub fn generate_gradient(
    start: Rgb,
    end: Rgb,
    direction: GradientDirection,
) -> [Rgb; TOTAL_ZONES] {
    let mut zones = [start; TOTAL_ZONES];

    for (z, zone_item) in zones.iter_mut().enumerate() {
        let r = (z / 4) as f32; // 0.0 .. 5.0 (6 rows)
        let c = (z % 4) as f32; // 0.0 .. 3.0 (4 cols)

        let factor = match direction {
            GradientDirection::Horizontal => c / 3.0_f32,
            GradientDirection::Vertical => r / 5.0_f32,
            GradientDirection::Diagonal => ((r / 5.0_f32) + (c / 3.0_f32)) / 2.0_f32,
            GradientDirection::Radial => {
                let dr = r - 2.5_f32;
                let dc = c - 1.5_f32;
                let dist = (dr * dr + dc * dc).sqrt();
                let max_dist = (2.5_f32 * 2.5_f32 + 1.5_f32 * 1.5_f32).sqrt();
                dist / max_dist
            }
        };

        *zone_item = lerp_rgb(start, end, factor);
    }

    zones
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp_rgb_endpoints() {
        let red = Rgb::new(255, 0, 0);
        let blue = Rgb::new(0, 0, 255);

        assert_eq!(lerp_rgb(red, blue, 0.0), red);
        assert_eq!(lerp_rgb(red, blue, 1.0), blue);
    }

    #[test]
    fn test_lerp_rgb_midpoint() {
        let black = Rgb::new(0, 0, 0);
        let white = Rgb::new(255, 255, 255);
        let mid = lerp_rgb(black, white, 0.5);

        assert_eq!(mid.r, 128);
        assert_eq!(mid.g, 128);
        assert_eq!(mid.b, 128);
    }

    #[test]
    fn test_horizontal_gradient() {
        let red = Rgb::new(255, 0, 0);
        let blue = Rgb::new(0, 0, 255);
        let g = generate_gradient(red, blue, GradientDirection::Horizontal);

        // Zone 0 is Col 0 (leftmost) -> should be pure red
        assert_eq!(g[0], red);
        // Zone 3 is Col 3 (rightmost) -> should be pure blue
        assert_eq!(g[3], blue);
        // Zone 4 is Col 0 (Row 1 leftmost) -> pure red
        assert_eq!(g[4], red);
        // Zone 7 is Col 3 (Row 1 rightmost) -> pure blue
        assert_eq!(g[7], blue);
    }

    #[test]
    fn test_vertical_gradient() {
        let red = Rgb::new(255, 0, 0);
        let blue = Rgb::new(0, 0, 255);
        let g = generate_gradient(red, blue, GradientDirection::Vertical);

        // Row 0 (Zones 0..3) -> pure red
        assert_eq!(g[0], red);
        assert_eq!(g[3], red);
        // Row 5 (Zones 20..23) -> pure blue
        assert_eq!(g[20], blue);
        assert_eq!(g[23], blue);
    }
}
