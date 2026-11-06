#!/bin/bash
set -euo pipefail

# Script to build and upload source packages to Launchpad PPA
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="1.2.0"
DISTROS=("noble" "jammy" "oracular")
PPA_TARGET="ppa:anas-kummali/msi-mysticlight"

echo "=== Building Debian Source Packages for Launchpad PPA ==="

WORKDIR="$(mktemp -d)"
trap 'rm -rf "${WORKDIR}"' EXIT

cd "${REPO_ROOT}"
# Create pristine upstream tarball
tar --exclude-vcs --exclude="./target" --exclude="./.git" -czf "${WORKDIR}/msi-mysticlight_${VERSION}.orig.tar.gz" -C "${REPO_ROOT}/.." "$(basename "${REPO_ROOT}")"

for DISTRO in "${DISTROS[@]}"; do
    echo "--- Preparing source package for Ubuntu ${DISTRO} ---"
    BUILD_DIR="${WORKDIR}/msi-mysticlight-${VERSION}-${DISTRO}"
    mkdir -p "${BUILD_DIR}"
    tar -xzf "${WORKDIR}/msi-mysticlight_${VERSION}.orig.tar.gz" --strip-components=1 -C "${BUILD_DIR}"

    # Setup debian folder
    cp -r "${REPO_ROOT}/packaging/debian" "${BUILD_DIR}/debian"

    # Update changelog distribution
    sed -i "1s/(.*) [a-z]*/(${VERSION}-1~${DISTRO}1) ${DISTRO}/" "${BUILD_DIR}/debian/changelog"

    cd "${BUILD_DIR}"
    if command -v dpkg-buildpackage >/dev/null 2>&1; then
        dpkg-buildpackage -S -sa -d
        echo "Source package for ${DISTRO} generated in ${WORKDIR}"
    else
        echo "dpkg-buildpackage not found on this host. Run inside Ubuntu/Debian or container."
    fi
done

echo ""
echo "=== Ready for upload ==="
echo "To upload to Launchpad PPA, run:"
echo "  dput ${PPA_TARGET} ${WORKDIR}/*_source.changes"

