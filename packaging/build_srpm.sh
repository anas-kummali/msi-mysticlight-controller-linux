#!/bin/bash
set -euo pipefail

# Script to generate Source RPM (.src.rpm) for Fedora COPR
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTDIR="${1:-${REPO_ROOT}/target/copr}"
VERSION="1.2.0"

echo "=== Building Source RPM for MSI Mystic Light v${VERSION} ==="
mkdir -p "${OUTDIR}"
WORKDIR="$(mktemp -d)"
trap 'rm -rf "${WORKDIR}"' EXIT

# Create source tarball
TAR_NAME="msi-mysticlight-${VERSION}"
mkdir -p "${WORKDIR}/${TAR_NAME}"

# Copy tracked repository files cleanly (excluding target/ and git metadata)
cd "${REPO_ROOT}"
git archive --format=tar --prefix="${TAR_NAME}/" HEAD | tar -xf - -C "${WORKDIR}"

# Create archive
tar -czf "${WORKDIR}/${TAR_NAME}.tar.gz" -C "${WORKDIR}" "${TAR_NAME}"

# Setup RPM build root
RPMTOP="${WORKDIR}/rpmbuild"
mkdir -p "${RPMTOP}/SOURCES" "${RPMTOP}/SPECS" "${RPMTOP}/SRPMS"

cp "${WORKDIR}/${TAR_NAME}.tar.gz" "${RPMTOP}/SOURCES/"
cp "${REPO_ROOT}/packaging/copr/msi-mysticlight.spec" "${RPMTOP}/SPECS/"

# Build Source RPM
rpmbuild --define "_topdir ${RPMTOP}" -bs "${RPMTOP}/SPECS/msi-mysticlight.spec"

# Copy generated SRPM to output directory
cp "${RPMTOP}/SRPMS/"*.src.rpm "${OUTDIR}/"

echo "=== Successfully built Source RPM ==="
ls -lh "${OUTDIR}/"*.src.rpm

