"""Constants and definitions for MSI MysticLight keyboard controllers."""

HIDIOCSFEATURE_64 = 0xC0404806
HIDIOCGFEATURE_64 = 0xC0404807

# Standard hardware animation modes
MODE_OFF = 0
MODE_STATIC = 1
MODE_BREATHING = 2
MODE_CYCLE = 3
MODE_WAVE = 4

WAVE_RTL = 0
WAVE_LTR = 1

# Supported Vendor IDs
VENDOR_MSI = "1462"
VENDOR_REALTEK_MSI = "0DB0"

# Product IDs known in the MSI MysticLight family
KNOWN_PIDS = {
    "1462:1603": "MSI Vector 16 HX AI (MS-1603) - 24-Zone",
    "0DB0:1605": "MSI Vector A18 HX (MS-1605) - 24-Zone",
    "0DB0:1801": "MSI Crosshair 16 Max HX (MS-1801) - 4-Zone",
    "0DB0:1606": "MSI Cyborg 15 (MS-1606) - 4-Zone",
    "1462:1601": "MSI Katana 15 HX (MS-1565) - 4-Zone",
    "1462:1562": "MSI Delta 15 (MS-1562) - 4-Zone",
    "1462:1563": "MSI Alpha 15/17 (MS-1563) - 4-Zone",
    "1462:1564": "MSI Stealth / Raider (MS-1564) - 4-Zone",
}

# Color palette presets
COLORS = {
    "white": (255, 255, 255),
    "warmwhite": (255, 220, 180),
    "red": (255, 0, 0),
    "green": (0, 255, 0),
    "blue": (0, 120, 255),
    "cyan": (0, 255, 255),
    "magenta": (255, 0, 255),
    "purple": (160, 32, 240),
    "yellow": (255, 220, 0),
    "orange": (255, 100, 0),
    "ice": (180, 220, 255),
}

# 24-zone physical matrix layout:
# 6 rows x 4 zones per row = 24 zones total
ROW_ZONES = {
    1: [0, 1, 2, 3],       # Row 1: ESC to DEL
    2: [4, 5, 6, 7],       # Row 2: ` to BACKSPACE
    3: [8, 9, 10, 11],     # Row 3: TAB to \
    4: [12, 13, 14, 15],   # Row 4: CAPS to ENTER
    5: [16, 17, 18, 19],   # Row 5: SHIFT to SHIFT
    6: [20, 21, 22, 23],   # Row 6: CTRL to CTRL / Space / Arrows / Numpad bottom
}
