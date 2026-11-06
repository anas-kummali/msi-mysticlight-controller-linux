"""Security and hardening test suite for MSI Mystic Light Controller for Linux.

Tests boundary defenses, input validation, path traversal resistance,
and buffer overflow protections against abuse cases.
"""

import os
import pytest
from unittest.mock import patch

from msi_keyboard.controller import find_device, parse_color, MSIKeyboardController
from msi_keyboard.protocol import (
    build_format_b_static_packet,
    build_msi_center_color_packet,
    build_format_a_packet,
    build_zone_mask_packet,
    build_effect_packet,
    _clamp_byte,
)


def test_clamp_byte_boundary_safety():
    """Verify numeric clamping prevents bytearray assignment overflows."""
    assert _clamp_byte(-100) == 0
    assert _clamp_byte(0) == 0
    assert _clamp_byte(255) == 255
    assert _clamp_byte(256) == 255
    assert _clamp_byte(999999) == 255
    assert _clamp_byte("invalid") == 0


def test_device_override_rejection_of_non_hidraw():
    """Verify find_device rejects regular files and non-hidraw paths."""
    # 1. Regular file attempt
    with patch.dict(os.environ, {"MSI_KEYBOARD_DEVICE": "/etc/passwd"}):
        dev = find_device()
        assert dev != "/etc/passwd"

    # 2. Block/character non-hidraw device attempt (/dev/null is char, but not hidraw)
    with patch.dict(os.environ, {"MSI_KEYBOARD_DEVICE": "/dev/null"}):
        dev = find_device()
        assert dev != "/dev/null"


def test_malformed_color_input_defense():
    """Verify color parsing rejects command injection and malformed hex."""
    malformed_inputs = [
        "; rm -rf /",
        "red; echo hacked",
        "#12",
        "#1234",
        "#1234567",
        "#GGGGGG",
        "../../etc/shadow",
        "0x123456",
        "",
        "   ",
    ]
    for bad in malformed_inputs:
        with pytest.raises(ValueError):
            parse_color(bad, 10)


def test_protocol_clamping_under_extreme_inputs():
    """Verify packet builders handle out-of-range integer inputs without crashing."""
    # Out of range RGB values
    pkt_b = build_format_b_static_packet(r=999, g=-50, b=1000, speed=9999)
    assert pkt_b[11] == 255
    assert pkt_b[12] == 0
    assert pkt_b[13] == 255

    pkt_c = build_msi_center_color_packet(r=500, g=-10, b=300, brightness_pct=999)
    assert pkt_c[11] == 255
    assert pkt_c[12] == 0
    assert pkt_c[13] == 255
    assert pkt_c[14] == 100

    pkt_a = build_format_a_packet(mode=1, speed=200, brightness=50, r=500, g=-10, b=255)
    assert pkt_a[4] == 10  # brightness clamped to 10 max
    assert pkt_a[6] == 255
    assert pkt_a[7] == 0


def test_zone_mask_bounds():
    """Verify zone mask packet asserts bounds strictly [0..31]."""
    assert build_zone_mask_packet(0)[2] == 0x01
    assert build_zone_mask_packet(23) is not None

    with pytest.raises(ValueError):
        build_zone_mask_packet(-1)

    with pytest.raises(ValueError):
        build_zone_mask_packet(32)


def test_controller_zone_and_row_enforcement():
    """Verify controller enforces 24-zone and 6-row physical matrix constraints."""
    ctrl = MSIKeyboardController(device_node="/dev/mock_hidraw")
    # Mock open and send_feature
    ctrl._fd = 999
    ctrl.send_feature = lambda data: None

    # Invalid zone >= 24
    with pytest.raises(ValueError, match="Invalid zone: 24"):
        ctrl.set_color(255, 255, 255, zone=24)

    with pytest.raises(ValueError, match="Invalid zone: -1"):
        ctrl.set_color(255, 255, 255, zone=-1)

    # Invalid row
    with pytest.raises(ValueError, match="Invalid row: 7"):
        ctrl.set_color(255, 255, 255, row=7)

    with pytest.raises(ValueError, match="Invalid row: 0"):
        ctrl.set_color(255, 255, 255, row=0)


def test_controller_fd_destructor_cleanup():
    """Verify __del__ safely calls close() without throwing exceptions."""
    ctrl = MSIKeyboardController(device_node="/dev/mock_hidraw")
    ctrl._fd = 999
    ctrl.close = lambda: setattr(ctrl, "_fd", None)
    ctrl.__del__()
    assert ctrl._fd is None
