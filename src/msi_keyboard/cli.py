"""Command line interface for msi-mysticlight / msi-keyboard / msi-rgb."""

from __future__ import annotations

import argparse
import sys

from msi_keyboard import __version__
from msi_keyboard.constants import (
    COLORS,
    MODE_BREATHING,
    MODE_CYCLE,
    MODE_WAVE,
    ROW_ZONES,
)
from msi_keyboard.controller import MSIKeyboardController, parse_color


def print_status(controller: MSIKeyboardController) -> None:
    info = controller.get_device_info()
    print("=" * 50)
    print(" MSI Mystic Light Linux Controller")
    print(f" Version : {__version__}")
    print("=" * 50)
    print(f"Device Node     : {info.get('node')}")
    print(f"Hardware Status : {'Online' if info.get('present') else 'Not Detected'}")
    print(f"Permissions     : {'Read/Write (OK)' if info.get('accessible') else 'Permission Denied (Check udev)'}")
    print(f"Total Zones     : {info.get('total_zones', 24)} Zones")
    print(f"Total Rows      : {info.get('total_rows', 6)} Physical Rows")
    print("\nPhysical Row Layout:")
    print("  - Row 1 (ESC to DEL)       : Zones 0..3")
    print("  - Row 2 (` to BACKSPACE)   : Zones 4..7")
    print("  - Row 3 (TAB to \\)         : Zones 8..11")
    print("  - Row 4 (CAPS to ENTER)    : Zones 12..15")
    print("  - Row 5 (SHIFT to SHIFT)   : Zones 16..19")
    print("  - Row 6 (CTRL to CTRL/Nav) : Zones 20..23")
    print("=" * 50)


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="msi-mysticlight",
        description="MSI Mystic Light Linux — Keyboard Backlight Controller (Vector 16 HX AI, Crosshair, Pulse, Katana, etc.)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""Examples:
  msi-mysticlight -c white -B 10       # Set all 6 rows to bright white
  msi-mysticlight -c "#FF8000"         # Set all rows to custom orange
  msi-mysticlight -c red -r 1          # Set only Row 1 (ESC to DEL) to red
  msi-mysticlight -w                   # Activate rainbow wave effect across all 6 rows
  msi-mysticlight -b -c cyan           # Activate cyan breathing effect
  msi-mysticlight -s                   # Activate color cycle effect
  msi-mysticlight -o                   # Power off backlight completely
  msi-mysticlight --status             # Show controller & device status
""",
    )
    parser.add_argument(
        "-c",
        "--color",
        default="white",
        help=f"Color name ({', '.join(COLORS.keys())}) or hex (#RRGGBB)",
    )
    parser.add_argument(
        "-B",
        "--brightness",
        type=int,
        default=10,
        choices=range(0, 11),
        metavar="[0..10]",
        help="Brightness level from 0 (off) to 10 (max, default: 10)",
    )
    parser.add_argument(
        "-o",
        "--off",
        action="store_true",
        help="Turn off keyboard backlight completely across all 6 rows",
    )
    parser.add_argument(
        "-w",
        "--wave",
        action="store_true",
        help="Run rainbow wave animation across all 6 rows",
    )
    parser.add_argument(
        "-b",
        "--breathe",
        action="store_true",
        help="Run breathing effect using selected color",
    )
    parser.add_argument(
        "-s",
        "--cycle",
        action="store_true",
        help="Run spectrum color cycle animation",
    )
    parser.add_argument(
        "-r",
        "--row",
        type=int,
        choices=range(1, 7),
        metavar="[1..6]",
        help="Target a specific physical row (1: Function, 2: Number, 3: Tab/QWERTY, 4: Caps/ASDF, 5: Shift/ZXCV, 6: Space/Ctrl)",
    )
    parser.add_argument(
        "-z",
        "--zone",
        type=int,
        choices=range(0, 24),
        metavar="[0..23]",
        help="Target a specific individual zone (0..23)",
    )
    parser.add_argument(
        "--status",
        action="store_true",
        help="Display detected hardware node, permission state, and zone map",
    )
    parser.add_argument(
        "-v",
        "--version",
        action="version",
        version=f"%(prog)s {__version__}",
    )

    args = parser.parse_args()

    try:
        ctrl = MSIKeyboardController()
    except Exception as e:
        print(f"Error initializing MSI keyboard controller: {e}", file=sys.stderr)
        sys.exit(1)

    if args.status:
        print_status(ctrl)
        return

    try:
        ctrl.open()
    except PermissionError as pe:
        print(f"Error: {pe}", file=sys.stderr)
        sys.exit(2)
    except Exception as e:
        print(f"Error opening keyboard device: {e}", file=sys.stderr)
        sys.exit(1)

    with ctrl:
        if args.off or args.brightness == 0:
            ctrl.turn_off()
            print("Keyboard backlight turned OFF across all 6 rows.")
            return

        if args.wave:
            rainbow = [
                (255, 0, 0),
                (255, 128, 0),
                (255, 255, 0),
                (0, 255, 0),
                (0, 255, 255),
                (0, 0, 255),
                (255, 0, 255),
            ]
            ctrl.set_effect(MODE_WAVE, rainbow, speed=200)
            print("Applied Rainbow Wave effect across all 6 rows (all 24 zones).")
            return

        if args.cycle:
            rainbow = [(255, 0, 0), (0, 255, 0), (0, 120, 255), (255, 255, 0)]
            ctrl.set_effect(MODE_CYCLE, rainbow, speed=150)
            print("Applied Color Cycle effect across all 6 rows (all 24 zones).")
            return

        try:
            raw_rgb = parse_color(args.color, 10)
        except ValueError as ve:
            print(f"Error: {ve}", file=sys.stderr)
            sys.exit(1)

        if args.breathe:
            ctrl.set_effect(MODE_BREATHING, [(0, 0, 0), raw_rgb, (0, 0, 0)], speed=120)
            print(f"Applied Breathing effect with {args.color} across all 6 rows.")
            return

        ctrl.set_color(
            *raw_rgb,
            brightness=args.brightness,
            row=args.row,
            zone=args.zone,
        )
        target = (
            f"Row {args.row}"
            if args.row
            else (f"Zone {args.zone}" if args.zone is not None else "All 6 Rows (24 Zones)")
        )
        print(
            f"Set static color to {args.color} (RGB: {raw_rgb}, Brightness: {args.brightness}/10) on {target}."
        )


if __name__ == "__main__":
    main()
