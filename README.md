# MSI Mystic Light Linux

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Python: 3.8+](https://img.shields.io/badge/Python-3.8%2B-brightgreen.svg)](https://www.python.org/)
[![Platform: Linux hidraw](https://img.shields.io/badge/Platform-Linux%20hidraw-orange.svg)](https://kernel.org/)
[![Hardware: 24--Zone RGB](https://img.shields.io/badge/Hardware-24--Zone%20RGB-purple.svg)]()

A robust, zero-dependency Linux command-line tool and driver for **MSI Gaming Laptop Keyboard Backlights** (including **MSI Vector 16 HX AI**, **Vector A18 HX**, **Crosshair 16**, **Katana**, **Pulse**, and **Raider** series).

This driver resolves the notorious **"Keyboard backlight only works in first 3 rows"** issue on Linux, providing complete control over all 6 physical rows, 24 individual zones, animations, and sleep/resume persistence.

---

## The Problem: The 3-Row Limitation on Linux

On modern MSI laptops equipped with multi-zone Mystic Light keyboards (e.g. USB HID `1462:1603` / `MSI MysticLight MS-1603`), standard Linux RGB tools only illuminate the top three rows:

* **Row 1:** `ESC` to `DEL` *(Illuminated)*
* **Row 2:** `` ` `` to `BACKSPACE` *(Illuminated)*
* **Row 3:** `TAB` to `\` *(Illuminated)*
* **Row 4:** `CAPS` to `ENTER` (**DARK / UNLIT**)
* **Row 5:** `SHIFT` to `SHIFT` (**DARK / UNLIT**)
* **Row 6:** `CTRL` to `CTRL` / `Space` / `Arrows` (**DARK / UNLIT**)

### Root Cause
1. **Legacy 4-Zone Mask Assumption:** Older utilities assumed 4 macro zones and sent a 4-bit zone mask `0x0F` (`0000 1111b`).
2. **24-Zone Physical Matrix:** The actual keyboard hardware controller multiplexes **6 physical rows $\times$ 4 zones per row = 24 discrete zones**.
3. **Firmware Filter:** Because mode packets and zone filters were limited to `0x0F`, zones 4 through 23 were permanently masked off by the onboard microcontroller.

---

## How This Driver Solves It

* **256-Bit Global Prep Packet:** Dispatches `[0x02, 0x01] + [0xFF] * 32` over `ioctl(HIDIOCSFEATURE)` to assert and configure all 24 zones simultaneously.
* **Full-Matrix Mode Packets:** Sets unmasked filter flags (`0xFF`) in Format B mode reports, broadcasting RGB parameters across all rows.
* **OEM MSI Center Hardware Slot:** Concurrently updates register slot `0x58`, matching OEM Windows software behavior.
* **Granular Control:** Supports full-board lighting, individual row targeting (`-r 1` through `-r 6`), and individual zone addressing (`-z 0` through `-z 23`).
* **Zero Dependencies:** Pure Python standard library interacting with Linux `/dev/hidraw*`. No background daemons, bloated runtimes, or CPU usage during static lighting or hardware-driven animations.

---

## Hardware Compatibility

| Model Series | USB VID:PID | HID Device Name | Keyboard Architecture | Status |
| :--- | :--- | :--- | :--- | :--- |
| **MSI Vector 16 HX AI (A2XWHG / MS-1603)** | `1462:1603` | `MSI MysticLight MS-1603` | **24-Zone Matrix (6 Rows $\times$ 4)** | **Verified** |
| **MSI Vector A18 HX (MS-1605)** | `0db0:1605` | `MSI MysticLight MS-1605` | **24-Zone Matrix (6 Rows $\times$ 4)** | Supported |
| **MSI Crosshair 16 Max HX (MS-1801)** | `0db0:1801` | `MSI MysticLight MS-1801` | 4-Zone / 24-Zone Matrix | Supported |
| **MSI Cyborg 15 (MS-1606)** | `0db0:1606` | `MSI MysticLight MS-1606` | 4-Zone RGB | Supported |
| **MSI Katana 15 HX (MS-1565)** | `1462:1601` | `MSI MysticLight MS-1565` | 4-Zone RGB | Supported |
| **MSI Delta 15 (MS-1562)** | `1462:1562` | `MSI MysticLight MS-1562` | 4-Zone RGB | Supported |
| **MSI Alpha 15 / 17 (MS-1563)** | `1462:1563` | `MSI MysticLight MS-1563` | 4-Zone RGB | Supported |

---

## Installation

### Method 1: Automated Installer (Recommended)

```bash
git clone https://github.com/anas-kummali/msi-mysticlight-linux.git
cd msi-mysticlight-linux
./install.sh
```

The installer will:
1. Install `msi-mysticlight` (with aliases `msi-keyboard` and `msi-rgb`) to `~/.local/bin/`.
2. Configure `/etc/udev/rules.d/99-msi-mysticlight.rules` for non-root hardware access.
3. Install systemd user service for persistence on boot and resume after suspend.

### Method 2: Python / Pipx

```bash
pip install .
# or for an isolated environment:
pipx install .
```

To configure non-root udev rules manually:
```bash
sudo cp packaging/99-msi-mysticlight.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

---

## Usage

### 1. Static Colors (Full Board)

```bash
# Set all 6 rows to solid white at maximum brightness (10/10)
msi-mysticlight -c white -B 10

# Set full board to custom hex color
msi-mysticlight -c "#00D4FF" -B 8

# Available color presets: white, warmwhite, red, green, blue, cyan, magenta, purple, yellow, orange, ice
msi-mysticlight -c ice
```

### 2. Row-by-Row Independent Lighting

Control individual keyboard rows independently:

```bash
# Set Row 1 (ESC to DEL) to Red
msi-mysticlight -c red -r 1

# Set Row 4 (CAPS to ENTER) to Green
msi-mysticlight -c green -r 4

# Set Row 6 (CTRL, Space, Arrows, Numpad) to Blue
msi-mysticlight -c blue -r 6
```

#### Physical Row Mapping
* **Row 1:** `ESC`, `F1`..`F12`, `DEL` *(Zones 0..3)*
* **Row 2:** `` ` ``, `1`..`0`, `-`, `=`, `BACKSPACE` *(Zones 4..7)*
* **Row 3:** `TAB`, `Q`..`P`, `[`, `]`, `\` *(Zones 8..11)*
* **Row 4:** `CAPS LOCK`, `A`..`L`, `;`, `'`, `ENTER` *(Zones 12..15)*
* **Row 5:** `LSHIFT`, `Z`..`/`, `RSHIFT` *(Zones 16..19)*
* **Row 6:** `LCTRL`, `FN`, `SUPER`, `ALT`, `SPACEBAR`, `RCTRL`, `ARROWS`, `NUMPAD` *(Zones 20..23)*

### 3. Hardware-Driven Animations

Animations are computed on-chip by the keyboard firmware, consuming **0% system CPU**:

```bash
# Rainbow Wave across all 6 rows
msi-mysticlight -w

# Breathing pulse with custom color
msi-mysticlight -b -c magenta

# Color Spectrum Cycle
msi-mysticlight -s
```

### 4. Turn Off Backlight

```bash
msi-mysticlight -o
# or set brightness to 0:
msi-mysticlight -B 0
```

### 5. Hardware Status & Diagnostics

```bash
msi-mysticlight --status
```

*Example Output:*
```text
==================================================
 MSI Mystic Light Linux Controller
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

## Suspend & Resume Persistence

When laptops sleep or enter suspend-to-RAM, USB bus power to the keyboard microcontroller is reset. The included systemd user service restores your configured backlight upon wake:

```bash
systemctl --user enable msi-keyboard-resume.service
```

---

## Testing & Quality

Run the test suite using `pytest`:

```bash
pytest
```

---

## License

This program is free software: you can redistribute it and/or modify it under the terms of the **GNU General Public License as published by the Free Software Foundation, version 3 (GPL-3.0)**.

See the [LICENSE](LICENSE) file for complete license text.
