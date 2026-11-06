//! MSI Mystic Light Controller for Linux (Library)
//!
//! High-performance, zero-external-dependency controller for MSI MysticLight
//! 24-Zone and 4-Zone keyboard backlights on Linux.

pub mod constants;
pub mod controller;
pub mod protocol;

pub const VERSION: &str = "1.0.0";

pub use constants::*;
pub use controller::{find_device, parse_color, DeviceInfo, MSIKeyboardController};
pub use protocol::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prep_packet() {
        let p = build_all_zones_prep_packet();
        assert_eq!(p.len(), 64);
        assert_eq!(p[0], 0x02);
        assert_eq!(p[1], 0x01);
        for i in 2..34 {
            assert_eq!(p[i], 0xFF, "byte {} must be 0xFF", i);
        }
        for i in 34..64 {
            assert_eq!(p[i], 0x00, "byte {} must be 0x00", i);
        }
    }

    #[test]
    fn test_zone_mask_packets() {
        let p0 = build_zone_mask_packet(0).unwrap();
        assert_eq!(&p0[2..6], &[0x01, 0x00, 0x00, 0x00]);

        let p16 = build_zone_mask_packet(16).unwrap();
        assert_eq!(&p16[2..6], &[0x00, 0x00, 0x01, 0x00]);

        let p23 = build_zone_mask_packet(23).unwrap();
        assert_eq!(&p23[2..6], &[0x00, 0x00, 0x80, 0x00]);

        assert!(build_zone_mask_packet(32).is_err());
    }

    #[test]
    fn test_format_b_static_packet() {
        let p = build_format_b_static_packet(255, 128, 0, 100, 0xFF);
        assert_eq!(p.len(), 64);
        assert_eq!(p[0], 0x02);
        assert_eq!(p[1], 0x02);
        assert_eq!(p[2], MODE_STATIC);
        assert_eq!(p[3], 100);
        assert_eq!(p[4], 0);
        assert_eq!(p[7], 0xFF);
        assert_eq!(p[10], 0);
        assert_eq!(&p[11..14], &[255, 128, 0]);
        assert_eq!(p[14], 100);
        assert_eq!(&p[15..18], &[255, 128, 0]);
    }

    #[test]
    fn test_msi_center_color_packet() {
        let p = build_msi_center_color_packet(0, 255, 120, 80, 0x58, 0x32, 0x08);
        assert_eq!(p.len(), 64);
        assert_eq!(p[0], 0x02);
        assert_eq!(p[1], 0x02);
        assert_eq!(p[2], MODE_STATIC);
        assert_eq!(p[3], 0x58);
        assert_eq!(p[6], 0x32);
        assert_eq!(p[7], 0x08);
        assert_eq!(&p[11..14], &[0, 255, 120]);
        assert_eq!(p[14], 80);
        assert_eq!(&p[15..18], &[0, 255, 120]);
    }

    #[test]
    fn test_format_a_packet() {
        let p = build_format_a_packet(MODE_STATIC, 0, 10, 255, 255, 255);
        assert_eq!(p.len(), 64);
        assert_eq!(p[0], 0x02);
        assert_eq!(p[1], 0x00);
        assert_eq!(p[2], MODE_STATIC);
        assert_eq!(p[3], 0);
        assert_eq!(p[4], 10);
        assert_eq!(p[5], 0x01);
        assert_eq!(&p[6..9], &[255, 255, 255]);
    }

    #[test]
    fn test_off_packet() {
        let p = build_off_packet();
        assert_eq!(p.len(), 64);
        assert_eq!(p[0], 0x02);
        assert_eq!(p[1], 0x02);
        assert_eq!(p[2], MODE_OFF);
        assert_eq!(p[3], 0x64);
        assert_eq!(p[7], 0xFF);
        assert_eq!(p[8], 0x01);
    }

    #[test]
    fn test_effect_packet() {
        let colors = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        let p = build_effect_packet(MODE_WAVE, &colors, 200, WAVE_LTR);
        assert_eq!(p.len(), 64);
        assert_eq!(p[0], 0x02);
        assert_eq!(p[1], 0x02);
        assert_eq!(p[2], MODE_WAVE);
        assert_eq!(p[3], 200);
        assert_eq!(p[7], 0xFF);
        assert_eq!(p[9], WAVE_LTR);

        // Keyframe 0 (0%): red
        assert_eq!(p[10], 0);
        assert_eq!(&p[11..14], &[255, 0, 0]);

        // Keyframe 1 (33%): green
        assert_eq!(p[14], 33);
        assert_eq!(&p[15..18], &[0, 255, 0]);

        // Keyframe 2 (66%): blue
        assert_eq!(p[18], 66);
        assert_eq!(&p[19..22], &[0, 0, 255]);

        // Closing keyframe (100%): red
        assert_eq!(p[22], 100);
        assert_eq!(&p[23..26], &[255, 0, 0]);
    }

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
    fn test_row_zones_mapping() {
        assert_eq!(ROW_ZONES.len(), 6);
        let mut seen = std::collections::HashSet::new();
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
    fn test_security_device_override_rejection() {
        // Set an invalid environment override pointing to /etc/passwd or non-existent
        unsafe {
            std::env::set_var("MSI_KEYBOARD_DEVICE", "/etc/passwd");
        }
        // /etc/passwd is a regular file, NOT a character device starting with /dev/hidraw
        let dev = find_device();
        if let Some(ref d) = dev {
            assert_ne!(d.to_str().unwrap(), "/etc/passwd");
            assert!(d.to_str().unwrap().starts_with("/dev/hidraw"));
        }
        unsafe {
            std::env::remove_var("MSI_KEYBOARD_DEVICE");
        }
    }
}
