import pytest
from msi_keyboard.protocol import (
    build_all_zones_prep_packet,
    build_zone_mask_packet,
    build_format_b_static_packet,
    build_msi_center_color_packet,
    build_format_a_packet,
    build_off_packet,
    build_effect_packet,
)
from msi_keyboard.constants import MODE_STATIC, MODE_OFF, MODE_WAVE


def test_build_all_zones_prep_packet():
    pkt = build_all_zones_prep_packet()
    assert len(pkt) == 64
    assert pkt[0] == 0x02
    assert pkt[1] == 0x01
    # Check that all 32 bytes (256 bits) are set to 0xFF
    assert pkt[2:34] == bytearray([0xFF] * 32)
    assert all(b == 0 for b in pkt[34:])


def test_build_zone_mask_packet():
    # Zone 0 -> mask = 1
    pkt0 = build_zone_mask_packet(0)
    assert pkt0[0] == 0x02
    assert pkt0[1] == 0x01
    assert pkt0[2] == 0x01
    assert pkt0[3] == 0x00

    # Zone 12 -> mask = 1 << 12 = 0x1000
    pkt12 = build_zone_mask_packet(12)
    assert pkt12[2] == 0x00
    assert pkt12[3] == 0x10

    # Zone 23 -> mask = 1 << 23 = 0x800000
    pkt23 = build_zone_mask_packet(23)
    assert pkt23[2] == 0x00
    assert pkt23[3] == 0x00
    assert pkt23[4] == 0x80

    with pytest.raises(ValueError):
        build_zone_mask_packet(32)


def test_build_format_b_static_packet():
    pkt = build_format_b_static_packet(255, 128, 0, speed=100, zone_mask=0xFF)
    assert len(pkt) == 64
    assert pkt[0] == 0x02
    assert pkt[1] == 0x02
    assert pkt[2] == MODE_STATIC
    assert pkt[3] == 100
    assert pkt[4] == 0
    assert pkt[7] == 0xFF  # Key assertion: all 24 zones enabled
    assert pkt[11:14] == bytearray([255, 128, 0])
    assert pkt[15:18] == bytearray([255, 128, 0])


def test_build_msi_center_color_packet():
    pkt = build_msi_center_color_packet(0, 255, 128, brightness_pct=80)
    assert len(pkt) == 64
    assert pkt[0] == 0x02
    assert pkt[1] == 0x02
    assert pkt[2] == MODE_STATIC
    assert pkt[3] == 0x58
    assert pkt[6] == 0x32
    assert pkt[7] == 0x08
    assert pkt[11:14] == bytearray([0, 255, 128])
    assert pkt[14] == 80  # Brightness percentage
    assert pkt[15:18] == bytearray([0, 255, 128])


def test_build_format_a_packet():
    pkt = build_format_a_packet(MODE_STATIC, speed=0, brightness=10, r=255, g=255, b=255)
    assert len(pkt) == 64
    assert pkt[0] == 0x02
    assert pkt[1] == 0x00
    assert pkt[2] == MODE_STATIC
    assert pkt[4] == 10
    assert pkt[6:9] == bytearray([255, 255, 255])


def test_build_off_packet():
    pkt = build_off_packet()
    assert len(pkt) == 64
    assert pkt[0] == 0x02
    assert pkt[1] == 0x02
    assert pkt[2] == MODE_OFF
    assert pkt[7] == 0xFF


def test_build_effect_packet():
    colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255)]
    pkt = build_effect_packet(mode=MODE_WAVE, colors=colors, speed=200)
    assert len(pkt) == 64
    assert pkt[0] == 0x02
    assert pkt[1] == 0x02
    assert pkt[2] == MODE_WAVE
    assert pkt[3] == 200
    assert pkt[7] == 0xFF
    # Keyframe 0 (red)
    assert pkt[11:14] == bytearray([255, 0, 0])
    # Keyframe 1 (green)
    assert pkt[15:18] == bytearray([0, 255, 0])
    # Closing loop wraps to red
    assert pkt[23:26] == bytearray([255, 0, 0])

