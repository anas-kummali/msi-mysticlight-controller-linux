# MSI 24-Zone Keyboard Backlight Linux Controller

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Python: 3.8+](https://img.shields.io/badge/Python-3.8%2B-brightgreen.svg)](https://www.python.org/)
[![Linux: Tested](https://img.shields.io/badge/Platform-Linux%20hidraw-orange.svg)](https://kernel.org/)

A lightweight, zero-dependency Linux command-line tool and driver for **MSI 24-Zone RGB Gaming Keyboards** (such as the **MSI Vector 16 HX AI**, **Vector A18 HX**, **Crosshair 16**, and **Pulse/Katana** series).

This tool fixes the infamous **"Keyboard backlight only works in the first 3 rows"** issue on Linux, restoring full-board illumination across all 6 physical rows.

---

## The Problem: The 3-Row Limitation on Linux

On modern MSI laptops equipped with 24-Zone RGB keyboards (e.g. `1462:1603`), existing Linux RGB utilities (such as OpenRGB or LightShow) only illuminate the top three rows of the keyboard:

* **Row 1:** `ESC` to `DEL` (illuminated)
* **Row 2:** `` ` `` to `BACKSPACE` (illuminated)
* **Row 3:** `TAB` to `\` (illuminated)
* **Row 4:** `CAPS` to `ENTER` (**DARK / UNLIT**)
* **Row 5:** `SHIFT` to `SHIFT` (**DARK / UNLIT**)
* **Row 6:** `CTRL` to `CTRL` / `Space` / `Arrows` (**DARK / UNLIT**)

### Root Cause
1. **4-Zone Assumption:** Legacy reverse-engineered tools assumed the keyboard had only 4 macro zones (like older Katana MS-1565 or Crosshair MS-1801 boards) and sent a 4-bit zone mask: `0x0F` (`0000 1111b`).
2. **24-Zone Matrix:** The hardware keyboard is actually organized as **6 physical rows $\times$ 4 zones per row = 24 zones**.
3. **Mask Cutoff:** In 24-zone hardware, bits 0..3 only correspond to the first block (Rows 1–3). Because higher-order mask bytes and mode packet filters (`p[7]`) were hardcoded to `0x0F`, zones 4 through 23 were permanently suppressed by the firmware.

---

## How This Tool Solves It

* **256-Bit Global Prep Packet:** Sends `[0x02, 0x01] + [0xFF] * 32` to select and prepare all 24 zones simultaneously.
* **Full-Zone Extended Mode Packets:** Clears the `0x0F` filter byte to `0xFF`, allowing the controller to broadcast color and hardware animation cycles to all zones.
* **Dual-Stack HID Transmission:** Concurrently dispatches Format B keyframes, OEM MSI Center packets (`0x58` profile slot), and Format A global hardware registers.
* **Individual Row & Zone Addressing:** Supports targeting specific rows (`-r 1` to `-r 6`) or individual zones (`-z 0` to `-z 23`).
* **Zero Dependencies:** Uses standard Linux Python with direct `ioctl(HIDIOCSFEATURE)` calls over `/dev/hidraw*`. No heavy frameworks, daemons, or background CPU usage.

---

## Hardware Compatibility

| Model | USB VID:PID | HID Device Name | Keyboard Architecture |
| :--- | :--- | :--- | :--- |
| **MSI Vector 16 HX AI (A2XWHG / MS-1603)** | `1462:1603` | `MSI MysticLight MS-1603` | **24-Zone RGB** (Verified) |
| **MSI Vector A18 HX (MS-1605)** | `0db0:1605` | `MSI MysticLight MS-1605` | **24-Zone RGB** |
| **MSI Crosshair 16 Max HX (MS-1801)** | `0db0:1801` | `MSI MysticLight MS-1801` | 4-Zone RGB |
| **MSI Cyborg 15 (MS-1606)** | `0db0:1606` | `MSI MysticLight MS-1606` | 4-Zone RGB |
| **MSI Katana 15 HX (MS-1565)** | `1462:1601` | `MSI MysticLight MS-1565` | 4-Zone RGB |
| **MSI Delta 15 (MS-1562)** | `1462:1562` | `MSI MysticLight MS-1562` | 4-Zone RGB |
| **MSI Alpha 15 / 17 (MS-1563)** | `1462:1563` | `MSI MysticLight MS-1563` | 4-Zone RGB |

---

## Installation

### Option 1: Quick Automated Install (Recommended)

Clone the repository and run `install.sh`:

```bash
git clone https://github.com/anas-kummali/msi-24zone-keyboard.git
cd msi-24zone-keyboard
./install.sh
```

The installer will:
1. Install `msi-keyboard` (and alias `msi-rgb`) to `~/.local/bin/`.
2. Install `/etc/udev/rules.d/99-msi-mysticlight.rules` so you can control lighting without `sudo`.
3. Enable systemd user service to restore your backlight settings across reboots and suspend/resume cycles.

### Option 2: Pip / Pipx Installation

```bash
pip install .
# or for isolated global installation:
pipx install .
```

Then copy the udev rule manually:
```bash
sudo cp packaging/99-msi-mysticlight.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

---

## Usage & Examples

### Static Colors (Full Board)

```bash
# Set all 6 rows to bright solid white (maximum brightness 10/10)
msi-keyboard -c white -B 10

# Set all 6 rows to custom hex color at 80% brightness
msi-keyboard -c "#FF8000" -B 8

# Available color presets: white, warmwhite, red, green, blue, cyan, magenta, purple, yellow, orange, ice
msi-keyboard -c ice
```

### Row-by-Row Targeting

You can illuminate individual physical rows independently:

```bash
# Set only Row 1 (ESC to DEL) to Red
msi-keyboard -c red -r 1

# Set Row 4 (CAPS to ENTER) to Green
msi-keyboard -c green -r 4

# Set Row 6 (CTRL / Space / Arrows) to Blue
msi-keyboard -c blue -r 6
```

#### Row Layout Reference
* **Row 1:** `ESC`, `F1`..`F12`, `DEL` (Zones 0..3)
* **Row 2:** `` ` ``, `1`..`0`, `-`, `=`, `BACKSPACE` (Zones 4..7)
* **Row 3:** `TAB`, `Q`..`P`, `[`, `]`, `\` (Zones 8..11)
* **Row 4:** `CAPS LOCK`, `A`..`L`, `;`, `'`, `ENTER` (Zones 12..15)
* **Row 5:** `LSHIFT`, `Z`..`/`, `RSHIFT` (Zones 16..19)
* **Row 6:** `LCTRL`, `FN`, `SUPER`, `ALT`, `SPACEBAR`, `RCTRL`, `ARROWS`, `NUMPAD` (Zones 20..23)

### Hardware Animated Effects

Because animations run directly on the keyboard microcontroller's internal timer, they consume **0% CPU**:

```bash
# Rainbow Wave across all 6 rows
msi-keyboard -w

# Breathing effect with custom color
msi-keyboard -b -c magenta

# Spectrum Color Cycle effect
msi-keyboard -s
```

### Turn Off Backlight

```bash
# Cut power to all keyboard LEDs across all 6 rows
msi-keyboard -o
# or
msi-keyboard -B 0
```

### Inspect Hardware & Zone Status

```bash
msi-keyboard --status
```

*Example Output:*
```text
==================================================
 MSI 24-Zone Keyboard Backlight Controller
 Version : 1.0.0
==================================================
Device Node     : /dev/hidraw3
Hardware Status : Online
Permissions     : Read/Write (OK)
Total Zones     : 24 Zones
Total Rows      : 6 Physical Rows

Physical Row Layout:
  - Row 1 (ESC to DEL)       : Zones 0..3
  - Row 2 (` to BACKSPACE)   : Zones 4..7
  - Row 3 (TAB to \)         : Zones 8..11
  - Row 4 (CAPS to ENTER)    : Zones 12..15
  - Row 5 (SHIFT to SHIFT)   : Zones 16..19
  - Row 6 (CTRL to CTRL/Nav) : Zones 20..23
==================================================
```

---

## Sleep & Resume Persistence

When your laptop suspends, USB power to the keyboard controller is cut. The included systemd user unit automatically restores your preferred backlight setting upon wake:

```bash
systemctl --user enable msi-keyboard-resume.service
```

---

## Troubleshooting

### "Permission denied on /dev/hidrawX"
Ensure the udev rule is installed and reloaded:
```bash
sudo cp packaging/99-msi-mysticlight.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```
If you still receive permission errors, log out and log back in, or check that your user account has access to `uaccess`.

---

## Contributing & Testing

Run unit tests with pytest:
```bash
pytest tests/
```

Pull requests and device reports for additional MSI laptop models are welcome!

---

## License

This project is licensed under the [MIT License](LICENSE).
