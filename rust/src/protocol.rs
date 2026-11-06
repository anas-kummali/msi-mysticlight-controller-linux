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
