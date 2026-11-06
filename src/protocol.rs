//! Low-level HID report packet builders for MSI MysticLight controllers.

use crate::constants::{MODE_OFF, MODE_STATIC};

/// Build a global prep packet asserting all 256 bits across all zone banks.
///
/// This ensures that 24-zone controllers (MS-1603, MS-1605) have all 6 physical
/// rows selected simultaneously before applying mode/color updates.
pub fn build_all_zones_prep_packet() -> [u8; 64] {
    let mut buf = [0u8; 64];
    buf[0] = 0x02; // Report ID 2
    buf[1] = 0x01; // Zone selection packet ID
    buf[2..34].fill(0xFF); // 256 bits set: selects every zone (0..23+)
    buf
}

/// Build a 32-bit zone mask packet for a single zone (0..31).
pub fn build_zone_mask_packet(zone_idx: u8) -> Result<[u8; 64], String> {
    if zone_idx > 31 {
        return Err(format!(
            "zone_idx must be an integer between 0 and 31 (got {})",
            zone_idx
        ));
    }
    let mask: u32 = 1 << zone_idx;
    let mut buf = [0u8; 64];
    buf[0] = 0x02;
    buf[1] = 0x01;
    buf[2] = (mask & 0xFF) as u8;
    buf[3] = ((mask >> 8) & 0xFF) as u8;
    buf[4] = ((mask >> 16) & 0xFF) as u8;
    buf[5] = ((mask >> 24) & 0xFF) as u8;
    Ok(buf)
}

/// Build an extended Format B static mode packet with unmasked zone flag.
pub fn build_format_b_static_packet(
    r: u8,
    g: u8,
    b: u8,
    speed: u16,
    zone_mask: u8,
) -> [u8; 64] {
    let mut p = [0u8; 64];
    p[0] = 0x02; // Report ID 2
    p[1] = 0x02; // Mode packet ID
    p[2] = MODE_STATIC; // Static Mode (1)
    p[3] = (speed & 0xFF) as u8; // Duration LSB (100 = 1.00s)
    p[4] = ((speed >> 8) & 0xFF) as u8; // Duration MSB
    p[7] = zone_mask; // Unmasked zone byte (0xFF enables all zones)
    p[8] = 0x01;
    p[9] = 0x00;
    p[10] = 0; // Keyframe 0 (0% of cycle)
    p[11] = r;
    p[12] = g;
    p[13] = b;
    p[14] = 100; // Keyframe 1 (100% of cycle)
    p[15] = r;
    p[16] = g;
    p[17] = b;
    p
}

/// Build the verified MSI Center OEM lighting packet for MS-1603 hardware.
pub fn build_msi_center_color_packet(
    r: u8,
    g: u8,
    b: u8,
    brightness_pct: u8,
    profile_slot: u8,
    transition: u8,
    effect_code: u8,
) -> [u8; 64] {
    let mut p = [0u8; 64];
    p[0] = 0x02;
    p[1] = 0x02;
    p[2] = MODE_STATIC;
    p[3] = profile_slot;
    p[4] = 0x02;
    p[5] = 0x00;
    p[6] = transition;
    p[7] = effect_code;
    p[8] = 0x01;
    p[9] = 0x01;
    p[10] = 0x00;
    p[11] = r;
    p[12] = g;
    p[13] = b;
    p[14] = brightness_pct.min(100);
    p[15] = r;
    p[16] = g;
    p[17] = b;
    p
}

/// Build a Format A universal motherboard/keyboard global lighting packet.
pub fn build_format_a_packet(
    mode: u8,
    speed: u8,
    brightness: u8,
    r: u8,
    g: u8,
    b: u8,
) -> [u8; 64] {
    let mut p = [0u8; 64];
    p[0] = 0x02;
    p[1] = 0x00;
    p[2] = mode;
    p[3] = speed;
    p[4] = brightness.min(10);
    p[5] = 0x01;
    p[6] = r;
    p[7] = g;
    p[8] = b;
    p
}

/// Build a Format B Mode 0 packet that truly cuts power to the keyboard LEDs.
pub fn build_off_packet() -> [u8; 64] {
    let mut p = [0u8; 64];
    p[0] = 0x02;
    p[1] = 0x02;
    p[2] = MODE_OFF;
    p[3] = 0x64;
    p[4] = 0x00;
    p[7] = 0xFF;
    p[8] = 0x01;
    p
}

/// Build hardware animated keyframe packet (Wave, Breathe, Cycle).
pub fn build_effect_packet(
    mode: u8,
    colors: &[(u8, u8, u8)],
    speed: u16,
    wave_dir: u8,
) -> [u8; 64] {
    let mut p = [0u8; 64];
    p[0] = 0x02;
    p[1] = 0x02;
    p[2] = mode;
    p[3] = (speed & 0xFF) as u8;
    p[4] = ((speed >> 8) & 0xFF) as u8;
    p[7] = 0xFF; // Full zone mask across 24 zones
    p[8] = 0x01;
    p[9] = wave_dir;

    if !colors.is_empty() {
        let step = (100 / colors.len().max(1)) as u8;
        for (i, c) in colors.iter().take(9).enumerate() {
            p[10 + i * 4] = (i as u8) * step;
            p[11 + i * 4] = c.0;
            p[12 + i * 4] = c.1;
            p[13 + i * 4] = c.2;
        }

        // Close animation cycle smoothly
        let idx = colors.len().min(9);
        p[10 + idx * 4] = 100;
        p[11 + idx * 4] = colors[0].0;
        p[12 + idx * 4] = colors[0].1;
        p[13 + idx * 4] = colors[0].2;
    }

    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::WAVE_LTR;

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
        let p = build_effect_packet(1, &colors, 200, WAVE_LTR);
        assert_eq!(p.len(), 64);
        assert_eq!(p[0], 0x02);
        assert_eq!(p[1], 0x02);
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
}
