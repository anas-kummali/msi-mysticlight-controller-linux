import pytest
from msi_keyboard.constants import ROW_ZONES
from msi_keyboard.controller import MSIKeyboardController, parse_color


def test_parse_color_named():
    assert parse_color("white", 10) == (255, 255, 255)
    assert parse_color("red", 10) == (255, 0, 0)
    assert parse_color("green", 10) == (0, 255, 0)
    assert parse_color("blue", 5) == (0, 60, 127)


def test_parse_color_hex():
    assert parse_color("#FFAA00", 10) == (255, 170, 0)
    assert parse_color("00FFFF", 10) == (0, 255, 255)
    with pytest.raises(ValueError):
        parse_color("foo_bar", 10)


def test_row_zones_mapping():
    assert len(ROW_ZONES) == 6
    all_zones = []
    for r in range(1, 7):
        assert len(ROW_ZONES[r]) == 4
        all_zones.extend(ROW_ZONES[r])
    assert all_zones == list(range(24))


def test_controller_mock_execution():
    written_reports = []

    class MockController(MSIKeyboardController):
        def open(self):
            self._fd = 999

        def close(self):
            self._fd = None

        def send_feature(self, data):
            written_reports.append(list(data))

    ctrl = MockController(device_node="/dev/mock_hidraw")
    with ctrl:
        # 1. Test set_color full board
        ctrl.set_color(255, 255, 255, brightness=10)
        assert len(written_reports) > 0
        # Check prep packet
        assert written_reports[0][0] == 0x02
        assert written_reports[0][1] == 0x01
        assert written_reports[0][2:34] == [0xFF] * 32

        # 2. Test turn_off
        written_reports.clear()
        ctrl.turn_off()
        assert len(written_reports) > 0
        # Check off packet mode
        assert any(pkt[1] == 0x02 and pkt[2] == 0x00 for pkt in written_reports)

        # 3. Test row targeting (Row 4: CAPS to ENTER)
        written_reports.clear()
        ctrl.set_color(255, 0, 0, row=4)
        assert len(written_reports) == 4 * 3  # 4 zones in row 4, each sends mask + p_b + p_c
