"""Low-level HID report packet builders for MSI MysticLight controllers."""

from __future__ import annotations
from typing import Sequence
from msi_keyboard.constants import (
    MODE_STATIC,
    MODE_OFF,
    WAVE_LTR,
)


def build_all_zones_prep_packet() -> bytearray:
    """Build a global prep packet asserting all 256 bits across all zone banks.
    
    This ensures that 24-zone controllers (MS-1603, MS-1605) have all 6 physical
    rows selected simultaneously before applying mode/color updates.
    """
    buf = bytearray(64)
    buf[0] = 0x02              # Report ID 2
    buf[1] = 0x01              # Zone selection packet ID
    buf[2:34] = b"\xFF" * 32   # 256 bits set: selects every zone (0..23+)
    return buf


def build_zone_mask_packet(zone_idx: int) -> bytearray:
    """Build a 32-bit zone mask packet for a single zone (0..23)."""
    if zone_idx < 0 or zone_idx > 31:
        raise ValueError(f"zone_idx must be between 0 and 31 (got {zone_idx})")
    mask = 1 << zone_idx
    buf = bytearray(64)
    buf[0] = 0x02
    buf[1] = 0x01
    buf[2] = mask & 0xFF
    buf[3] = (mask >> 8) & 0xFF
    buf[4] = (mask >> 16) & 0xFF
    buf[5] = (mask >> 24) & 0xFF
    return buf


def build_format_b_static_packet(
    r: int,
    g: int,
    b: int,
    speed: int = 100,
    zone_mask: int = 0xFF,
) -> bytearray:
    """Build an extended Format B static mode packet with unmasked zone flag."""
    p = bytearray(64)
    p[0] = 0x02                # Report ID 2
    p[1] = 0x02                # Mode packet ID
    p[2] = MODE_STATIC         # Static Mode (1)
    p[3] = speed & 0xFF        # Duration LSB (100 = 1.00s)
    p[4] = (speed >> 8) & 0xFF # Duration MSB
    p[7] = zone_mask & 0xFF    # Unmasked zone byte (0xFF enables all zones)
    p[8] = 0x01
    p[9] = 0x00
    p[10] = 0                  # Keyframe 0 (0% of cycle)
    p[11] = r
    p[12] = g
    p[13] = b
    p[14] = 100                # Keyframe 1 (100% of cycle)
    p[15] = r
    p[16] = g
    p[17] = b
    return p


def build_msi_center_color_packet(
    r: int,
    g: int,
    b: int,
    brightness_pct: int = 100,
    profile_slot: int = 0x58,
    transition: int = 0x32,
    effect_code: int = 0x08,
) -> bytearray:
    """Build the verified MSI Center OEM lighting packet for MS-1603 hardware."""
    p = bytearray(64)
    p[0] = 0x02
    p[1] = 0x02
    p[2] = MODE_STATIC
    p[3] = profile_slot & 0xFF
    p[4] = 0x02
    p[5] = 0x00
    p[6] = transition & 0xFF
    p[7] = effect_code & 0xFF
    p[8] = 0x01
    p[9] = 0x01
    p[10] = 0x00
    p[11] = r
    p[12] = g
    p[13] = b
    p[14] = max(0, min(100, brightness_pct))
    p[15] = r
    p[16] = g
    p[17] = b
    return p


def build_format_a_packet(
    mode: int,
    speed: int,
    brightness: int,
    r: int,
    g: int,
    b: int,
) -> bytearray:
    """Build a Format A universal motherboard/keyboard global lighting packet."""
    p = bytearray(64)
    p[0] = 0x02
    p[1] = 0x00
    p[2] = mode & 0xFF
    p[3] = speed & 0xFF
    p[4] = max(0, min(10, brightness))
    p[5] = 0x01
    p[6] = r
    p[7] = g
    p[8] = b
    return p


def build_off_packet() -> bytearray:
    """Build a Format B Mode 0 packet that truly cuts power to the keyboard LEDs."""
    p = bytearray(64)
    p[0] = 0x02
    p[1] = 0x02
    p[2] = MODE_OFF
    p[3] = 0x64
    p[4] = 0x00
    p[7] = 0xFF
    p[8] = 0x01
    return p


def build_effect_packet(
    mode: int,
    colors: Sequence[tuple[int, int, int]],
    speed: int = 150,
    wave_dir: int = WAVE_LTR,
) -> bytearray:
    """Build hardware animated keyframe packet (Wave, Breathe, Cycle)."""
    p = bytearray(64)
    p[0] = 0x02
    p[1] = 0x02
    p[2] = mode & 0xFF
    p[3] = speed & 0xFF
    p[4] = (speed >> 8) & 0xFF
    p[7] = 0xFF              # Full zone mask across 24 zones
    p[8] = 0x01
    p[9] = wave_dir & 0xFF

    if colors:
        step = 100 // max(1, len(colors))
        for i, c in enumerate(colors[:9]):
            p[10 + i * 4] = i * step
            p[11 + i * 4] = c[0]
            p[12 + i * 4] = c[1]
            p[13 + i * 4] = c[2]

        # Close animation cycle smoothly
        idx = min(9, len(colors))
        p[10 + idx * 4] = 100
        p[11 + idx * 4] = colors[0][0]
        p[12 + idx * 4] = colors[0][1]
        p[13 + idx * 4] = colors[0][2]

    return p
