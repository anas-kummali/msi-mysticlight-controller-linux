#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
UDEV_DIR="/etc/udev/rules.d"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"

if [[ -z "${BIN_DIR:-}" || -z "${SCRIPT_DIR:-}" ]]; then
    echo "Error: Critical installation directories are undefined." >&2
    exit 1
fi

echo "=== MSI Mystic Light Controller for Linux Installer (Rust Edition) ==="

# 1. Check Cargo / Rust toolchain
if ! command -v cargo &>/dev/null; then
    echo "Error: 'cargo' is not found in PATH." >&2
    echo "Please install Rust (e.g., 'curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh' or via your distribution package manager)." >&2
    exit 1
fi

# 2. Ensure bin dir exists
mkdir -p "${BIN_DIR}"

# 3. Compile and install optimized release binary
echo "[1/4] Compiling optimized Rust release binary..."
cargo build --release --manifest-path "${SCRIPT_DIR}/Cargo.toml" --quiet
cp -f "${SCRIPT_DIR}/target/release/msi-mysticlight" "${BIN_DIR}/msi-mysticlight"
chmod 755 "${BIN_DIR}/msi-mysticlight"
ln -sf "${BIN_DIR}/msi-mysticlight" "${BIN_DIR}/msi-keyboard"
ln -sf "${BIN_DIR}/msi-mysticlight" "${BIN_DIR}/msi-rgb"
echo "      Installed native Rust binary to ${BIN_DIR}/msi-mysticlight (564 KB)."

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
    echo "Test commands:"
    echo "  msi-mysticlight -c white -B 10"
    echo "  msi-mysticlight -w"
    echo "  msi-mysticlight --status"
    echo "  (Aliases 'msi-keyboard' and 'msi-rgb' are also available)"
else
    echo ""
    echo "Notice: Controller installed. If permission was denied, log out and back in for udev rules to apply."
fi
