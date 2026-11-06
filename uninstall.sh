#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="${HOME}/.local/bin"
UDEV_DIR="/etc/udev/rules.d"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"

if [[ -z "${BIN_DIR:-}" || -z "${SYSTEMD_USER_DIR:-}" ]]; then
    echo "Error: Critical environment directories are undefined." >&2
    exit 1
fi

echo "=== Uninstalling MSI Mystic Light Controller for Linux ==="

# 1. Stop and disable systemd services
if command -v systemctl &>/dev/null; then
    systemctl --user disable --now msi-keyboard.service 2>/dev/null || true
    systemctl --user disable --now msi-keyboard-resume.service 2>/dev/null || true
fi
rm -f "${SYSTEMD_USER_DIR}/msi-keyboard.service"
rm -f "${SYSTEMD_USER_DIR}/msi-keyboard-resume.service"

if command -v systemctl &>/dev/null; then
    systemctl --user daemon-reload 2>/dev/null || true
fi

# 2. Remove binaries and symlinks
rm -f "${BIN_DIR}/msi-mysticlight"
rm -f "${BIN_DIR}/msi-keyboard"
rm -f "${BIN_DIR}/msi-rgb"

# 3. Remove udev rule if requested
if [ -w "${UDEV_DIR}/99-msi-mysticlight.rules" ]; then
    rm -f "${UDEV_DIR}/99-msi-mysticlight.rules"
    udevadm control --reload-rules && udevadm trigger
elif command -v sudo &>/dev/null && [ -f "${UDEV_DIR}/99-msi-mysticlight.rules" ]; then
    sudo rm -f "${UDEV_DIR}/99-msi-mysticlight.rules"
    sudo udevadm control --reload-rules && sudo udevadm trigger
fi

echo "Uninstallation complete."
