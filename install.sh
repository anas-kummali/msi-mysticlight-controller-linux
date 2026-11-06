#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"
UDEV_DIR="/etc/udev/rules.d"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"

if [[ -z "${BIN_DIR:-}" || -z "${SCRIPT_DIR:-}" ]]; then
    echo "Error: Critical installation directories are undefined." >&2
    exit 1
fi

echo "=== MSI Mystic Light Controller & GUI Installer (v1.2.0) ==="

# 1. Check Cargo / Rust toolchain
if ! command -v cargo &>/dev/null; then
    echo "Error: 'cargo' is not found in PATH." >&2
    echo "Please install Rust (e.g., 'curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh' or via your distribution package manager)." >&2
    exit 1
fi

# 2. Ensure dirs exist
mkdir -p "${BIN_DIR}" "${APP_DIR}" "${ICON_DIR}"

# 3. Compile and install release binaries
echo "[1/4] Compiling release binaries (CLI & Native GUI)..."
cargo build --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet
cp -f "${SCRIPT_DIR}/target/release/msi-mysticlight" "${BIN_DIR}/msi-mysticlight"
cp -f "${SCRIPT_DIR}/target/release/msi-mysticlight-gui" "${BIN_DIR}/msi-mysticlight-gui"
chmod 755 "${BIN_DIR}/msi-mysticlight" "${BIN_DIR}/msi-mysticlight-gui"
ln -sf "${BIN_DIR}/msi-mysticlight" "${BIN_DIR}/msi-keyboard"
ln -sf "${BIN_DIR}/msi-mysticlight" "${BIN_DIR}/msi-rgb"

# Install Desktop Entry and Vector Icon
cp -f "${SCRIPT_DIR}/packaging/msi-mysticlight.desktop" "${APP_DIR}/"
cp -f "${SCRIPT_DIR}/packaging/icons/msi-mysticlight.svg" "${ICON_DIR}/"
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database -q "${APP_DIR}" 2>/dev/null || true
fi
echo "      Installed 'msi-mysticlight' (CLI) and 'msi-mysticlight-gui' (GUI)."
echo "      Registered desktop launcher and vector icon in application menu."

# 4. Install udev rule for non-root access
echo "[2/4] Setting up udev rules for non-root USB access..."
if [ -f "${UDEV_DIR}/99-msi-mysticlight.rules" ] && grep -q "1603" "${UDEV_DIR}/99-msi-mysticlight.rules" 2>/dev/null; then
    echo "      Udev rule already active in ${UDEV_DIR}/99-msi-mysticlight.rules."
elif [ -w "${UDEV_DIR}" ]; then
    cp "${SCRIPT_DIR}/packaging/99-msi-mysticlight.rules" "${UDEV_DIR}/"
    udevadm control --reload-rules && udevadm trigger
elif command -v sudo &>/dev/null; then
    echo "      Requesting sudo to install /etc/udev/rules.d/99-msi-mysticlight.rules..."
    sudo cp "${SCRIPT_DIR}/packaging/99-msi-mysticlight.rules" "${UDEV_DIR}/"
    sudo udevadm control --reload-rules && sudo udevadm trigger
else
    echo "      Warning: Could not install udev rule (no root/sudo access)."
    echo "      You may need to manually copy packaging/99-msi-mysticlight.rules to ${UDEV_DIR}/."
fi

# 5. Install systemd user service for auto-start and wake resume
echo "[3/4] Installing systemd user services..."
mkdir -p "${SYSTEMD_USER_DIR}"
cp "${SCRIPT_DIR}/packaging/systemd/msi-keyboard.service" "${SYSTEMD_USER_DIR}/"
cp "${SCRIPT_DIR}/packaging/systemd/msi-keyboard-resume.service" "${SYSTEMD_USER_DIR}/"

if command -v systemctl &>/dev/null; then
    systemctl --user daemon-reload || true
    systemctl --user enable msi-keyboard.service 2>/dev/null || true
    systemctl --user enable msi-keyboard-resume.service 2>/dev/null || true
fi

# 6. Test execution
echo "[4/4] Verifying keyboard hardware status..."
if "${BIN_DIR}/msi-mysticlight" --status; then
    echo ""
    echo "Initializing backlight to solid bright white across all 6 rows..."
    "${BIN_DIR}/msi-mysticlight" -c white -B 10 || true
    echo ""
    echo "=== Installation Successful! ==="
    echo "Make sure ${BIN_DIR} is in your PATH."
    echo ""
    echo "Available commands & tools:"
    echo "  msi-mysticlight-gui       # Launch native graphical user interface"
    echo "  msi-mysticlight --gui     # Launch GUI from CLI"
    echo "  msi-mysticlight -c white  # Set static color across all 24 zones"
    echo "  msi-mysticlight -w        # Activate rainbow wave effect"
    echo "  msi-mysticlight --status  # Inspect hardware connectivity"
    echo ""
    echo "The application is also accessible in your Desktop Application Menu as 'MSI Mystic Light'."
else
    echo ""
    echo "Notice: Controller installed. If permission was denied, log out and back in for udev rules to apply."
fi
