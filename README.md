<div align="center">

# ⚡ MSI Mystic Light Controller for Linux

### *High-Performance, Zero-Dependency Pure Rust Driver for MSI Laptop RGB Keyboards*

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg?style=for-the-badge)](LICENSE)
[![Rust: 2021](https://img.shields.io/badge/Rust-2021%20Edition-DEA584.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform: Linux hidraw](https://img.shields.io/badge/Platform-Linux%20hidraw-FCC624.svg?style=for-the-badge&logo=linux&logoColor=black)](https://kernel.org/)
[![Binary Size](https://img.shields.io/badge/Binary-564%20KB-2ea44f.svg?style=for-the-badge)]()
[![Dependencies](https://img.shields.io/badge/Dependencies-Zero%20(Pure%20Stdlib)-blueviolet.svg?style=for-the-badge)]()
[![Hardware](https://img.shields.io/badge/Hardware-24--Zone%20RGB-E05D44.svg?style=for-the-badge)]()
[![CI Build](https://img.shields.io/badge/CI-Passing%20(Multi--Distro)-brightgreen.svg?style=for-the-badge&logo=github-actions&logoColor=white)](.github/workflows/ci.yml)

<p align="center">
  <a href="#-the-problem-the-notorious-3-row-linux-limitation">The 3-Row Limitation</a> •
  <a href="#-architecture--solution">Architecture</a> •
  <a href="#-hardware-matrix--compatibility">Compatibility</a> •
  <a href="#-installation">Installation</a> •
  <a href="#-usage--examples">Usage & Examples</a> •
  <a href="#-systemd-persistence">Persistence</a> •
  <a href="#-security--hardening">Security</a> •
  <a href="#-benchmarks--testing">Testing</a>
</p>

---

</div>

## 📖 Overview

**MSI Mystic Light Controller for Linux** is an ultra-fast, standalone hardware driver and CLI written in **100% pure Rust standard library**. It directly communicates with the onboard keyboard microcontroller via POSIX `ioctl` feature reports on `/dev/hidraw*`.

It solves the widespread **"Keyboard backlight only lights up the first 3 rows"** bug that affects modern MSI gaming laptops running Linux, restoring full independent lighting to **all 6 physical rows and 24 discrete zones**.

---

## 🔍 The Problem: The Notorious 3-Row Linux Limitation

On modern MSI laptops featuring multi-zone Mystic Light RGB keyboards (such as the **Vector 16 HX AI**, **Vector A18 HX**, **Crosshair 16**, and **Pulse/Katana** series), standard Linux utilities (e.g. OpenRGB or legacy Python scripts) only illuminate the top three rows:

```text
┌────────────────────────────────────────────────────────┐
│ [Row 1] ESC  F1  F2  ...  F12  PRTSCR  DEL  [LIT   🟢] │
│ [Row 2]  `    1   2  ...   0     -      =   [LIT   🟢] │
│ [Row 3] TAB   Q   W  ...   P     [      ]   [LIT   🟢] │
│ [Row 4] CAPS  A   S  ...   L     ;    ENTER [DARK  🔴] │ <--- UNLIT
│ [Row 5] SHIFT Z   X  ...   /   SHIFT        [DARK  🔴] │ <--- UNLIT
│ [Row 6] CTRL  FN WIN ALT SPACE ARROWS NUM   [DARK  🔴] │ <--- UNLIT
└────────────────────────────────────────────────────────┘
```

### Root Cause Analysis
1. **Legacy 4-Zone Assumption:** Older tools assumed MSI keyboards were simple 4-macro-zone peripherals and dispatched a 4-bit zone mask `0x0F` (`0000 1111b`).
2. **24-Zone Multiplexed Matrix:** Modern hardware controllers multiplex **6 physical rows $\times$ 4 zones per row = 24 discrete lighting zones**.
3. **Firmware Register Lockout:** Because packets sent mask `0x0F`, hardware zones 4 through 23 were filtered out and permanently disabled by the keyboard microcontroller.

---

## 🛠️ Architecture & Solution

This driver circumvents firmware masking through an exact reverse-engineered protocol pipeline:

```mermaid
flowchart TD
    subgraph Userspace ["User Space (564 KB Rust Standalone Binary)"]
        CLI["msi-mysticlight CLI"] --> Parser["Zero-Crate Argument Parser"]
        Parser --> Controller["MSIKeyboardController"]
        Controller --> PacketBuilder["Stack-Allocated Packets [u8; 64]"]
    end

    subgraph Kernel ["Linux Kernel Subsystem"]
        PacketBuilder -->|"POSIX ioctl(HIDIOCSFEATURE_64)"| HidrawNode["/dev/hidraw0 (Character Device)"]
        HidrawNode --> USBHID["hid-generic / usbhid subsystem"]
    end

    subgraph Hardware ["MSI Laptop Keyboard Microcontroller"]
        USBHID -->|"64-Byte USB HID Report"| MCU["MysticLight MCU (MS-1603 / MS-1605)"]
        MCU --> Step1["1. 256-Bit Zone Assertion (0xFF * 32)"]
        MCU --> Step2["2. Format B Broadcast (Zone Mask 0xFF)"]
        MCU --> Step3["3. Slot 0x58 OEM Synchronizer"]
        MCU --> Step4["4. Per-Zone Register Pulse (Zones 0..23)"]
        Step4 --> FullMatrix["✨ All 6 Rows (24 Zones) Fully Illuminated"]
    end
```

### Feature Comparison Matrix

| Feature | Legacy Tools (OpenRGB / Python) | **MSI Mystic Light Controller for Linux** |
| :--- | :--- | :--- |
| **Row Coverage** | Top 3 rows only (Rows 4–6 unlit) | **All 6 Physical Rows fully illuminated** |
| **Zone Architecture** | 4-zone macro mask (`0x0F`) | **Full 24-zone physical matrix addressing** |
| **Dependencies** | Python 3, Qt, C++ runtime libraries | **Pure Rust standard library (0 external crates)** |
| **Binary Size** | 20 MB – 150 MB (or full runtime) | **564 KB standalone native ELF binary** |
| **Execution Latency** | 200 ms – 1500 ms | **< 1 ms user-space execution** |
| **Compilation Speed** | Several minutes | **< 0.3 seconds (Offline / Air-gapped)** |
| **CPU Overhead** | Continuous if background daemon | **0.0% CPU (on-chip microcontroller execution)** |
| **Sleep / Wake Persistence** | Reset to dark / unlit | **Automated systemd user resume service** |
| **Access Security** | Insecure world-writable `0666` udev | **Least-privilege `0660` + `uaccess` & path sanitization** |

---

## 💻 Hardware Matrix & Compatibility

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                 24-ZONE PHYSICAL MATRIX                                │
├────────┬──────────────────────────────────────┬───────────────────────┬────────────────┤
│ Row    │ Physical Key Mapping                 │ Zone Index Range      │ Hardware State │
├────────┼──────────────────────────────────────┼───────────────────────┼────────────────┤
│ Row 1  │ ESC, F1 - F12, PRTSCR, DEL           │ Zones 0,  1,  2,  3   │ Full RGB       │
│ Row 2  │ ~, 1 - 0, -, =, BACKSPACE            │ Zones 4,  5,  6,  7   │ Full RGB       │
│ Row 3  │ TAB, Q - P, [, ], \                  │ Zones 8,  9, 10, 11   │ Full RGB       │
│ Row 4  │ CAPS LOCK, A - L, ;, ', ENTER        │ Zones 12, 13, 14, 15  │ Full RGB       │
│ Row 5  │ LSHIFT, Z - M, ,, ., /, RSHIFT       │ Zones 16, 17, 18, 19  │ Full RGB       │
│ Row 6  │ LCTRL, FN, SUPER, ALT, SPACE, NAV    │ Zones 20, 21, 22, 23  │ Full RGB       │
└────────┴──────────────────────────────────────┴───────────────────────┴────────────────┘
```

### Verified Hardware Models

| Laptop Series | USB VID:PID | HID Device Name | Keyboard Matrix | Verified Status |
| :--- | :--- | :--- | :--- | :--- |
| **MSI Vector 16 HX AI (A2XWHG / MS-1603)** | `1462:1603` | `MSI MysticLight MS-1603` | **24-Zone Matrix (6 Rows $\times$ 4)** | **Verified (Tested)** |
| **MSI Vector A18 HX (MS-1605)** | `0db0:1605` | `MSI MysticLight MS-1605` | **24-Zone Matrix (6 Rows $\times$ 4)** | Supported |
| **MSI Crosshair 16 Max HX (MS-1801)** | `0db0:1801` | `MSI MysticLight MS-1801` | 4-Zone / 24-Zone Matrix | Supported |
| **MSI Cyborg 15 (MS-1606)** | `0db0:1606` | `MSI MysticLight MS-1606` | 4-Zone RGB | Supported |
| **MSI Katana 15 HX (MS-1565)** | `1462:1601` | `MSI MysticLight MS-1565` | 4-Zone RGB | Supported |
| **MSI Delta 15 (MS-1562)** | `1462:1562` | `MSI MysticLight MS-1562` | 4-Zone RGB | Supported |
| **MSI Alpha 15 / 17 (MS-1563)** | `1462:1563` | `MSI MysticLight MS-1563` | 4-Zone RGB | Supported |

---

## 🚀 Installation

### Option 1: Automated Installer (Recommended)

Clone the repository and run the automated installer:

```bash
git clone https://github.com/anas-kummali/msi-mysticlight-controller-linux.git
cd msi-mysticlight-controller-linux
./install.sh
```

**What the installer does automatically:**
1. Compiles the optimized release binary via `cargo build --release` (564 KB).
2. Installs `msi-mysticlight` to `~/.local/bin/` with aliases `msi-keyboard` and `msi-rgb`.
3. Installs `/etc/udev/rules.d/99-msi-mysticlight.rules` for non-root hardware access (`0660` + `uaccess`).
4. Configures systemd user units for auto-start on boot and restore after system suspend.
5. Verifies hardware connectivity and lights up all 6 rows to bright solid white.

### Option 2: Manual Cargo Build

Because this driver requires **no external crates**, it compiles offline in less than one second:

```bash
cargo build --release
cp target/release/msi-mysticlight ~/.local/bin/
ln -sf ~/.local/bin/msi-mysticlight ~/.local/bin/msi-keyboard
ln -sf ~/.local/bin/msi-mysticlight ~/.local/bin/msi-rgb

# Configure non-root udev permissions:
sudo cp packaging/99-msi-mysticlight.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

---

## 🎮 Usage & Examples

### 1. Static Full-Board Illumination

```bash
# Set all 6 physical rows to bright solid white (Default: 10/10)
msi-mysticlight -c white -B 10

# Custom hex colors:
msi-mysticlight -c "#00D4FF" -B 8
msi-mysticlight -c "#FF8000"

# Available built-in palette presets:
# white, warmwhite, red, green, blue, cyan, magenta, purple, yellow, orange, ice
msi-mysticlight -c ice
```

### 2. Independent Row Lighting (Row 1 through Row 6)

Control each physical row independently:

```bash
msi-mysticlight -c red     -r 1   # Row 1: ESC to DEL
msi-mysticlight -c green   -r 2   # Row 2: Numbers & Backspace
msi-mysticlight -c blue    -r 3   # Row 3: TAB to \
msi-mysticlight -c yellow  -r 4   # Row 4: CAPS to ENTER
msi-mysticlight -c magenta -r 5   # Row 5: SHIFT to SHIFT
msi-mysticlight -c cyan    -r 6   # Row 6: Space, Arrows & Numpad
```

### 3. Individual Zone Targeting (Zone 0 through Zone 23)

```bash
# Target individual physical zones (0..23):
msi-mysticlight -c red -z 0
msi-mysticlight -c cyan -z 23
```

### 4. Hardware-Driven Animations (0.0% CPU)

Animations run on-chip inside the keyboard microcontroller:

```bash
# Rainbow Wave effect across all 6 rows:
msi-mysticlight -w

# Breathing pulse with custom color:
msi-mysticlight -b -c magenta

# Color Spectrum Cycle:
msi-mysticlight -s
```

### 5. Turn Off Backlight

```bash
msi-mysticlight -o
# or set brightness to 0:
msi-mysticlight -B 0
```

### 6. Hardware Diagnostics & Status Report

```bash
msi-mysticlight --status
```

*Example Output:*
```text
==================================================
 MSI Mystic Light Controller for Linux
 Version : 1.0.0
==================================================
Device Node     : /dev/hidraw0
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

## ⚡ Systemd Persistence

When laptops enter suspend-to-RAM (`s2idle` or `deep`), USB bus power to the keyboard microcontroller is reset. The included systemd user service restores your configured backlight instantly upon waking:

```bash
systemctl --user enable --now msi-keyboard.service
systemctl --user enable --now msi-keyboard-resume.service
```

---

## 🛡️ Security & Hardening

This driver was audited and hardened under strict security principles:

1. **Least-Privilege Udev Rules:**
   * Uses `MODE="0660", GROUP="plugdev", TAG+="uaccess"`.
   * Never uses insecure world-writable `0666` permissions.
2. **Device Path Traversal Mitigation:**
   * Resolves canonical device paths via `std::fs::canonicalize`.
   * Enforces that device nodes strictly reside within `/dev/`, begin with `hidraw`, and assert `meta.file_type().is_char_device()`.
   * Blocks arbitrary block device manipulation or symbolic link attacks.
3. **Hardened Systemd Sandboxing:**
   * Units run with `NoNewPrivileges=yes`, `PrivateTmp=yes`, `ProtectControlGroups=yes`, `ProtectKernelTunables=yes`, and `RestrictAddressFamilies=AF_UNIX`.

---

## 🧪 Benchmarks & Testing

### Test Suite (`cargo test`)
The test suite validates packet accuracy, row/zone mappings, color parsing, and path security:

```bash
cargo test
```
```text
running 11 tests
test tests::test_prep_packet ... ok
test tests::test_zone_mask_packets ... ok
test tests::test_format_b_static_packet ... ok
test tests::test_msi_center_color_packet ... ok
test tests::test_format_a_packet ... ok
test tests::test_off_packet ... ok
test tests::test_effect_packet ... ok
test tests::test_parse_color_named ... ok
test tests::test_parse_color_hex ... ok
test tests::test_row_zones_mapping ... ok
test tests::test_security_device_override_rejection ... ok

running 7 integration tests
test test_cli_help ... ok
test test_cli_version ... ok
test test_cli_status ... ok
test test_cli_invalid_color ... ok
test test_cli_invalid_brightness ... ok
test test_cli_invalid_row ... ok
test test_cli_invalid_zone ... ok

test result: ok. 18 passed; 0 failed; finished in 0.05s
```

### Multi-Distribution Verification Matrix

Tested and verified in isolated containers without modifying host state:

| Linux Distribution | Kernel / Architecture | Test Result |
| :--- | :--- | :--- |
| **Ubuntu 26.10 (Stonking Stingray)** | Linux x86_64 (Rolling glibc) | **✓ Passed (18/18)** |
| **Ubuntu 26.04.1 LTS (Resolute Raccoon)** | Linux x86_64 (LTS) | **✓ Passed (18/18)** |
| **Debian Sid (Trixie / Unstable)** | Linux x86_64 | **✓ Passed (18/18)** |
| **Arch Linux** | Linux x86_64 (Pacman Rolling) | **✓ Passed (18/18)** |
| **openSUSE Tumbleweed** | Linux x86_64 (Zypper Rolling) | **✓ Passed (18/18)** |
| **Fedora Linux 45 (COSMIC)** | Linux x86_64 (Host Environment) | **✓ Passed (Live Hardware)** |

Run the container test suite locally using Podman or Docker:
```bash
./scripts/test_distros.sh
# Run with live hardware passthrough:
./scripts/test_distros.sh --live
```

---

## 📄 License

This program is free software: you can redistribute it and/or modify it under the terms of the **GNU General Public License as published by the Free Software Foundation, version 3 (GPL-3.0)**.

See the [LICENSE](LICENSE) file for the full license text.

---

<div align="center">
  <sub>Developed by <b>Anas Kummali</b> • Dedicated to Linux Hardware Liberation 🐧</sub>
</div>
