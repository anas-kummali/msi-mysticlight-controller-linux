#!/usr/bin/env bash
# Build Debian (.deb), RedHat/Fedora (.rpm), and Generic (.tar.gz) distribution packages.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VERSION="1.1.0"
OUT_DIR="${ROOT_DIR}/target/packages"

echo "=== Packaging MSI Mystic Light v${VERSION} ==="
mkdir -p "${OUT_DIR}"

# 1. Compile optimized binaries
echo "[1/4] Compiling release binaries..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

BIN_CLI="${ROOT_DIR}/target/release/msi-mysticlight"
BIN_GUI="${ROOT_DIR}/target/release/msi-mysticlight-gui"
strip "${BIN_CLI}" "${BIN_GUI}"

# 2. Build Debian Package (.deb)
echo "[2/4] Assembling Debian (.deb) package..."
DEB_DIR="$(mktemp -d /tmp/msi-mysticlight-deb.XXXXXX)"
trap 'rm -rf "${DEB_DIR}"' EXIT

mkdir -p "${DEB_DIR}/debian"
mkdir -p "${DEB_DIR}/data/usr/bin"
mkdir -p "${DEB_DIR}/data/etc/udev/rules.d"
mkdir -p "${DEB_DIR}/data/usr/lib/systemd/user"
mkdir -p "${DEB_DIR}/data/usr/share/applications"
mkdir -p "${DEB_DIR}/data/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${DEB_DIR}/data/usr/share/doc/msi-mysticlight"

# Install files into data tree
cp "${BIN_CLI}" "${DEB_DIR}/data/usr/bin/msi-mysticlight"
cp "${BIN_GUI}" "${DEB_DIR}/data/usr/bin/msi-mysticlight-gui"
ln -sf msi-mysticlight "${DEB_DIR}/data/usr/bin/msi-keyboard"
ln -sf msi-mysticlight "${DEB_DIR}/data/usr/bin/msi-rgb"

cp "${SCRIPT_DIR}/99-msi-mysticlight.rules" "${DEB_DIR}/data/etc/udev/rules.d/99-msi-mysticlight.rules"
cp "${SCRIPT_DIR}/systemd/msi-keyboard.service" "${DEB_DIR}/data/usr/lib/systemd/user/"
cp "${SCRIPT_DIR}/systemd/msi-keyboard-resume.service" "${DEB_DIR}/data/usr/lib/systemd/user/"
cp "${SCRIPT_DIR}/msi-mysticlight.desktop" "${DEB_DIR}/data/usr/share/applications/"
cp "${SCRIPT_DIR}/icons/msi-mysticlight.svg" "${DEB_DIR}/data/usr/share/icons/hicolor/scalable/apps/"
cp "${ROOT_DIR}/README.md" "${DEB_DIR}/data/usr/share/doc/msi-mysticlight/"
cp "${ROOT_DIR}/LICENSE" "${DEB_DIR}/data/usr/share/doc/msi-mysticlight/copyright"

# Create debian-binary
echo "2.0" > "${DEB_DIR}/debian-binary"

# Calculate installed size in KB
INST_SIZE=$(du -sk "${DEB_DIR}/data" | cut -f1)

# Control file
cat << EOF > "${DEB_DIR}/control"
Package: msi-mysticlight
Version: ${VERSION}
Architecture: amd64
Maintainer: Anas Kummali <anas@milkbun.in>
Section: utils
Priority: optional
Installed-Size: ${INST_SIZE}
Homepage: https://github.com/anas-kummali/msi-mysticlight-controller-linux
Description: MSI Mystic Light Controller & GUI for Linux
 High-performance native Rust controller and GUI application for MSI
 MysticLight 24-zone and 4-zone keyboard backlights. Integrates OpenRGB
 and LightShow features into a single unified desktop experience.
EOF

# Postinst script
cat << 'EOF' > "${DEB_DIR}/postinst"
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    udevadm control --reload-rules 2>/dev/null || true
    udevadm trigger 2>/dev/null || true
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
fi
exit 0
EOF
chmod 0755 "${DEB_DIR}/postinst"

# Postrm script
cat << 'EOF' > "${DEB_DIR}/postrm"
#!/bin/sh
set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    udevadm control --reload-rules 2>/dev/null || true
    udevadm trigger 2>/dev/null || true
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
fi
exit 0
EOF
chmod 0755 "${DEB_DIR}/postrm"

# Package control.tar.gz
(cd "${DEB_DIR}" && tar -czf control.tar.gz --owner=0 --group=0 control postinst postrm)

# Package data.tar.gz
(cd "${DEB_DIR}/data" && tar -czf "${DEB_DIR}/data.tar.gz" --owner=0 --group=0 .)

# Assemble deb
DEB_FILE="${OUT_DIR}/msi-mysticlight_${VERSION}_amd64.deb"
ar rcs "${DEB_FILE}" "${DEB_DIR}/debian-binary" "${DEB_DIR}/control.tar.gz" "${DEB_DIR}/data.tar.gz"
echo "      Built Debian package: ${DEB_FILE} ($(du -h "${DEB_FILE}" | cut -f1))"

# 3. Build RPM Package (.rpm)
echo "[3/4] Building Fedora/RedHat (.rpm) package..."
RPM_BUILD_DIR="$(mktemp -d /tmp/msi-mysticlight-rpm.XXXXXX)"
mkdir -p "${RPM_BUILD_DIR}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

cp "${BIN_CLI}" "${RPM_BUILD_DIR}/SOURCES/msi-mysticlight"
cp "${BIN_GUI}" "${RPM_BUILD_DIR}/SOURCES/msi-mysticlight-gui"
cp "${SCRIPT_DIR}/99-msi-mysticlight.rules" "${RPM_BUILD_DIR}/SOURCES/"
cp "${SCRIPT_DIR}/systemd/msi-keyboard.service" "${RPM_BUILD_DIR}/SOURCES/"
cp "${SCRIPT_DIR}/systemd/msi-keyboard-resume.service" "${RPM_BUILD_DIR}/SOURCES/"
cp "${SCRIPT_DIR}/msi-mysticlight.desktop" "${RPM_BUILD_DIR}/SOURCES/"
cp "${SCRIPT_DIR}/icons/msi-mysticlight.svg" "${RPM_BUILD_DIR}/SOURCES/"
cp "${SCRIPT_DIR}/msi-mysticlight.spec" "${RPM_BUILD_DIR}/SPECS/"

rpmbuild --define "_topdir ${RPM_BUILD_DIR}" -bb "${RPM_BUILD_DIR}/SPECS/msi-mysticlight.spec" >/dev/null

GENERATED_RPM=$(find "${RPM_BUILD_DIR}/RPMS" -name "*.rpm" | head -n1)
cp "${GENERATED_RPM}" "${OUT_DIR}/"
FINAL_RPM="${OUT_DIR}/$(basename "${GENERATED_RPM}")"
echo "      Built RPM package: ${FINAL_RPM} ($(du -h "${FINAL_RPM}" | cut -f1))"
rm -rf "${RPM_BUILD_DIR}"

# 4. Build Generic Tarball
echo "[4/4] Creating standalone (.tar.gz) distribution archive..."
TAR_DIR="$(mktemp -d /tmp/msi-mysticlight-tar.XXXXXX)"
TAR_PKG_DIR="${TAR_DIR}/msi-mysticlight-${VERSION}"
mkdir -p "${TAR_PKG_DIR}"/bin "${TAR_PKG_DIR}"/packaging

cp "${BIN_CLI}" "${TAR_PKG_DIR}/bin/msi-mysticlight"
cp "${BIN_GUI}" "${TAR_PKG_DIR}/bin/msi-mysticlight-gui"
cp -r "${SCRIPT_DIR}"/* "${TAR_PKG_DIR}/packaging/"
cp "${ROOT_DIR}/install.sh" "${TAR_PKG_DIR}/"
cp "${ROOT_DIR}/uninstall.sh" "${TAR_PKG_DIR}/"
cp "${ROOT_DIR}/README.md" "${TAR_PKG_DIR}/"
cp "${ROOT_DIR}/LICENSE" "${TAR_PKG_DIR}/"

TAR_FILE="${OUT_DIR}/msi-mysticlight-${VERSION}-x86_64-linux.tar.gz"
tar -czf "${TAR_FILE}" -C "${TAR_DIR}" "msi-mysticlight-${VERSION}"
echo "      Built Tarball archive: ${TAR_FILE} ($(du -h "${TAR_FILE}" | cut -f1))"
rm -rf "${TAR_DIR}"

echo ""
echo "=== Distribution Packages Generated Successfully ==="
ls -lh "${OUT_DIR}"
echo ""
echo "Checksums (SHA-256):"
sha256sum "${OUT_DIR}"/*
