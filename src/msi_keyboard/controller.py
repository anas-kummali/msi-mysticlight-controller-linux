"""High-level hardware controller interface for MSI 24-Zone & 4-Zone Keyboards."""

from __future__ import annotations

import fcntl
import glob
import os
import stat
import sys
import time
from typing import Sequence

from msi_keyboard.constants import (
    COLORS,
    HIDIOCSFEATURE_64,
    KNOWN_PIDS,
    MODE_BREATHING,
    MODE_CYCLE,
    MODE_OFF,
    MODE_STATIC,
    MODE_WAVE,
    ROW_ZONES,
    VENDOR_MSI,
    VENDOR_REALTEK_MSI,
)
from msi_keyboard.protocol import (
    build_all_zones_prep_packet,
    build_effect_packet,
    build_format_a_packet,
    build_format_b_static_packet,
    build_msi_center_color_packet,
    build_off_packet,
    build_zone_mask_packet,
)


def find_device() -> str | None:
    """Scan /sys/class/hidraw/ for MSI MysticLight USB hardware nodes."""
    # Check env override first with strict path and device type validation
    override = os.environ.get("MSI_KEYBOARD_DEVICE")
    if override:
        resolved = os.path.realpath(override)
        try:
            st = os.stat(resolved)
            # Must be a character device under /dev/ and match hidraw naming
            if (
                stat.S_ISCHR(st.st_mode)
                and resolved.startswith("/dev/")
                and os.path.basename(resolved).startswith("hidraw")
            ):
                return resolved
        except OSError:
            pass

    candidates: list[tuple[str, str, str]] = []
    for path in sorted(glob.glob("/sys/class/hidraw/hidraw*")):
        uevent_path = os.path.join(path, "device", "uevent")
        try:
            with open(uevent_path, "r", encoding="utf-8") as f:
                content = f.read()
        except OSError:
            continue

        up = content.upper()
        ident = ""
        name = ""
        for line in content.splitlines():
            if line.startswith("HID_ID="):
                bits = line.split("=", 1)[1].split(":")
                if len(bits) == 3:
                    ident = f"{bits[1][-4:].upper()}:{bits[2][-4:].upper()}"
            elif line.startswith("HID_NAME="):
                name = line.split("=", 1)[1].strip()

        has_vendor = VENDOR_MSI in up or VENDOR_REALTEK_MSI in up
        if ("MYSTICLIGHT" in up.replace(" ", "") or ident in KNOWN_PIDS) and has_vendor:
            node = "/dev/" + os.path.basename(path)
            if os.path.exists(node):
                candidates.append((node, ident, name))

    if not candidates:
        # Fallback to /dev/hidraw3 if present and is a char device
        if os.path.exists("/dev/hidraw3"):
            return "/dev/hidraw3"
        return None

    # Prefer 1462:1603 or known models
    candidates.sort(key=lambda d: (d[1] != "1462:1603", d[1] not in KNOWN_PIDS))
    return candidates[0][0]


def parse_color(val: str, brightness: int = 10) -> tuple[int, int, int]:
    """Parse color name or hex (#RRGGBB) and scale by brightness level [0..10]."""
    val = val.strip().lower()
    if val in COLORS:
        r, g, b = COLORS[val]
    else:
        h = val.lstrip("#")
        if len(h) == 3:
            h = "".join(c * 2 for c in h)
        if len(h) != 6:
            raise ValueError(
                f"Invalid color: '{val}'. Use hex (#RRGGBB) or named color "
                f"({', '.join(COLORS.keys())})"
            )
        try:
            r = int(h[0:2], 16)
            g = int(h[2:4], 16)
            b = int(h[4:6], 16)
        except ValueError as e:
            raise ValueError(f"Invalid hexadecimal color value: '{val}'") from e

    scale = max(0.0, min(1.0, float(brightness) / 10.0))
    return int(r * scale), int(g * scale), int(b * scale)


class MSIKeyboardController:
    """Manages raw HID communication with the MSI Keyboard backlight controller."""

    def __init__(self, device_node: str | None = None) -> None:
        self.device_node = device_node or find_device()
        self._fd: int | None = None

    def open(self) -> None:
        if not self.device_node:
            raise RuntimeError("MSI MysticLight HID device not found in /sys/class/hidraw/.")
        try:
            self._fd = os.open(self.device_node, os.O_RDWR)
        except PermissionError as e:
            raise PermissionError(
                f"Permission denied on {self.device_node}. "
                "Ensure udev rule /etc/udev/rules.d/99-msi-mysticlight.rules is installed."
            ) from e

    def close(self) -> None:
        if self._fd is not None:
            try:
                os.close(self._fd)
            except OSError:
                pass
            self._fd = None

    def __del__(self) -> None:
        self.close()

    def __enter__(self) -> MSIKeyboardController:
        self.open()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        self.close()

    def send_feature(self, data: Sequence[int] | bytes | bytearray) -> None:
        if self._fd is None:
            raise RuntimeError("Device handle is not open.")
        payload = bytes(data).ljust(64, b"\x00")[:64]
        try:
            fcntl.ioctl(self._fd, HIDIOCSFEATURE_64, payload)
        except OSError as e:
            raise OSError(
                f"Failed sending USB HID feature report to {self.device_node}: {e}"
            ) from e

    def set_color(
        self,
        r: int,
        g: int,
        b: int,
        brightness: int = 10,
        row: int | None = None,
        zone: int | None = None,
    ) -> None:
        """Set static color across all 6 rows (all 24 zones), a specific row, or a specific zone."""
        scale = max(0.0, min(1.0, float(brightness) / 10.0))
        sr, sg, sb = int(r * scale), int(g * scale), int(b * scale)
        b_pct = int(max(0, min(100, brightness * 10)))

        p_b = build_format_b_static_packet(sr, sg, sb, speed=100, zone_mask=0xFF)
        p_c = build_msi_center_color_packet(sr, sg, sb, brightness_pct=b_pct)
        p_a = build_format_a_packet(MODE_STATIC, speed=0, brightness=brightness, r=sr, g=sg, b=sb)

        if zone is not None:
            if not isinstance(zone, int) or zone < 0 or zone >= 24:
                raise ValueError(f"Invalid zone: {zone}. Must be between 0 and 23.")
            self.send_feature(build_zone_mask_packet(zone))
            self.send_feature(p_b)
            self.send_feature(p_c)
        elif row is not None:
            if row not in ROW_ZONES:
                raise ValueError(f"Invalid row: {row}. Supported rows: 1..6")
            for z in ROW_ZONES[row]:
                self.send_feature(build_zone_mask_packet(z))
                self.send_feature(p_b)
                self.send_feature(p_c)
                time.sleep(0.002)
        else:
            # Full 24-Zone, 6-Row simultaneous activation
            self.send_feature(build_all_zones_prep_packet())
            time.sleep(0.005)
            self.send_feature(p_b)
            time.sleep(0.005)
            self.send_feature(p_c)
            time.sleep(0.005)
            self.send_feature(p_a)
            time.sleep(0.005)

            # Ensure every physical LED register is updated
            for z in range(24):
                self.send_feature(build_zone_mask_packet(z))
                self.send_feature(p_b)
                self.send_feature(p_c)
                time.sleep(0.002)

    def turn_off(self) -> None:
        """Completely power off keyboard LEDs across all 6 rows and all 24 zones."""
        self.send_feature(build_all_zones_prep_packet())
        p_b = build_off_packet()
        p_a = build_format_a_packet(MODE_OFF, speed=0, brightness=0, r=0, g=0, b=0)

        self.send_feature(p_b)
        self.send_feature(p_a)

        for z in range(24):
            self.send_feature(build_zone_mask_packet(z))
            self.send_feature(p_b)
            time.sleep(0.002)

    def set_effect(
        self,
        mode: int,
        colors: Sequence[tuple[int, int, int]],
        speed: int = 150,
    ) -> None:
        """Apply hardware-interpolated effects across all 24 zones and all 6 rows."""
        self.send_feature(build_all_zones_prep_packet())
        time.sleep(0.01)

        p = build_effect_packet(mode=mode, colors=colors, speed=speed)
        self.send_feature(p)
        time.sleep(0.01)

        for z in range(24):
            self.send_feature(build_zone_mask_packet(z))
            self.send_feature(p)
            time.sleep(0.002)

    def get_device_info(self) -> dict[str, str | int | bool]:
        node = self.device_node
        if not node or not os.path.exists(node):
            return {"node": node or "None", "present": False}

        readable = os.access(node, os.R_OK | os.W_OK)
        return {
            "node": node,
            "present": True,
            "accessible": readable,
            "total_zones": 24,
            "total_rows": 6,
        }
