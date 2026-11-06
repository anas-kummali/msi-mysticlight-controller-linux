#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${HOME}/.local/bin"
UDEV_DIR="/etc/udev/rules.d"
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"

echo "=== MSI 24-Zone Keyboard Linux Installer ==="

# 1. Ensure bin dir exists
mkdir -p "${BIN_DIR}"

# 2. Install executable wrapper
echo "[1/4] Installing executable to ${BIN_DIR}/msi-keyboard..."
cat << 'EOF' > "${BIN_DIR}/msi-keyboard"
#!/usr/bin/env python3
import sys
import os

# Add package directory to path if running in-tree
current_dir = os.path.dirname(os.path.realpath(__file__))
repo_src = os.path.abspath(os.path.join(current_dir, "../../src"))
if os.path.isdir(repo_src) and repo_src not in sys.path:
    sys.path.insert(0, repo_src)

from msi_keyboard.cli import main

if __name__ == "__main__":
    main()
EOF
chmod +x "${BIN_DIR}/msi-keyboard"
ln -sf "${BIN_DIR}/msi-keyboard" "${BIN_DIR}/msi-rgb"

# Also install python package if pip/uv available
if command -v pip &>/dev/null; then
    echo "      Installing Python package via pip..."
    python3 -m pip install -e "${SCRIPT_DIR}" --quiet || true
elif command -v uv &>/dev/null; then
    echo "      Installing Python package via uv..."
    uv pip install -e "${SCRIPT_DIR}" --quiet || true
fi

# 3. Install udev rule
echo "[2/4] Setting up udev rules for non-root USB access..."
if [ -w "${UDEV_DIR}" ]; then
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

# 4. Install systemd user service for auto-start and wake resume
echo "[3/4] Installing systemd user services..."
mkdir -p "${SYSTEMD_USER_DIR}"
cp "${SCRIPT_DIR}/packaging/systemd/msi-keyboard.service" "${SYSTEMD_USER_DIR}/"
cp "${SCRIPT_DIR}/packaging/systemd/msi-keyboard-resume.service" "${SYSTEMD_USER_DIR}/"

if command -v systemctl &>/dev/null; then
    systemctl --user daemon-reload || true
    systemctl --user enable msi-keyboard.service 2>/dev/null || true
fi

# 5. Test execution
echo "[4/4] Verifying keyboard hardware status..."
if "${BIN_DIR}/msi-keyboard" --status; then
    echo ""
    echo "Initializing backlight to solid bright white across all 6 rows..."
    "${BIN_DIR}/msi-keyboard" -c white -B 10 || true
    echo ""
    echo "=== Installation Successful! ==="
    echo "Make sure ${BIN_DIR} is in your PATH."
    echo "Test commands:"
    echo "  msi-keyboard -c white -B 10"
    echo "  msi-keyboard -w"
    echo "  msi-keyboard --status"
else
    echo ""
    echo "Notice: Controller installed. If permission was denied, log out and back in for udev rules to apply."
fi
