"""MSI 24-Zone Keyboard Backlight Linux Controller."""

__version__ = "1.0.0"

from msi_keyboard.constants import COLORS, KNOWN_PIDS, ROW_ZONES
from msi_keyboard.controller import MSIKeyboardController, find_device, parse_color

__all__ = [
    "__version__",
    "COLORS",
    "KNOWN_PIDS",
    "ROW_ZONES",
    "MSIKeyboardController",
    "find_device",
    "parse_color",
]
