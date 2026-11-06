//! Physical keyboard matrix layout and zone mappings.

/// Total number of physical lighting rows on MSI 24-zone hardware.
pub const TOTAL_ROWS: usize = 6;

/// Total number of addressable hardware lighting zones.
pub const TOTAL_ZONES: usize = 24;

/// Number of discrete LED zones per physical row.
pub const ZONES_PER_ROW: usize = 4;

/// 24-zone physical matrix layout:
/// 6 physical rows x 4 zones per row = 24 zones total.
pub const ROW_ZONES: &[(u8, [u8; 4])] = &[
    (1, [0, 1, 2, 3]),       // Row 1: ESC to DEL
    (2, [4, 5, 6, 7]),       // Row 2: ` to BACKSPACE
    (3, [8, 9, 10, 11]),     // Row 3: TAB to \
    (4, [12, 13, 14, 15]),   // Row 4: CAPS to ENTER
    (5, [16, 17, 18, 19]),   // Row 5: SHIFT to SHIFT
    (6, [20, 21, 22, 23]),   // Row 6: CTRL to CTRL / Space / Arrows / Numpad bottom
];

/// Returns the 4 zone indices corresponding to a physical row [1..6].
pub fn get_row_zones(row: u8) -> Option<&'static [u8; 4]> {
    ROW_ZONES.iter().find(|(r, _)| *r == row).map(|(_, z)| z)
}

/// Returns whether a row index is in the valid range [1..6].
pub fn is_valid_row(row: u8) -> bool {
    (1..=6).contains(&row)
}

/// Returns whether a zone index is in the valid range [0..23].
pub fn is_valid_zone(zone: u8) -> bool {
    zone < 24
}

/// Find which physical row [1..6] contains the given zone index [0..23].
pub fn row_for_zone(zone: u8) -> Option<u8> {
    if !is_valid_zone(zone) {
        return None;
    }
    ROW_ZONES
        .iter()
        .find(|(_, zones)| zones.contains(&zone))
        .map(|(r, _)| *r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_row_zones_mapping() {
        assert_eq!(ROW_ZONES.len(), 6);
        let mut seen = HashSet::new();
        for (row, zones) in ROW_ZONES {
            assert!(*row >= 1 && *row <= 6);
            assert_eq!(zones.len(), 4);
            for &z in zones {
                assert!(z < 24);
                assert!(seen.insert(z), "Duplicate zone {}", z);
            }
        }
        assert_eq!(seen.len(), 24);
    }

    #[test]
    fn test_row_lookups() {
        assert_eq!(get_row_zones(1), Some(&[0, 1, 2, 3]));
        assert_eq!(get_row_zones(6), Some(&[20, 21, 22, 23]));
        assert_eq!(get_row_zones(0), None);
        assert_eq!(get_row_zones(7), None);

        assert!(is_valid_row(1));
        assert!(is_valid_row(6));
        assert!(!is_valid_row(0));
        assert!(!is_valid_row(7));

        assert!(is_valid_zone(0));
        assert!(is_valid_zone(23));
        assert!(!is_valid_zone(24));

        assert_eq!(row_for_zone(0), Some(1));
        assert_eq!(row_for_zone(5), Some(2));
        assert_eq!(row_for_zone(23), Some(6));
        assert_eq!(row_for_zone(24), None);
    }
}

