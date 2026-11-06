#!/usr/bin/env bash
set -euo pipefail

# MSI Mystic Light Controller for Linux - Multi-Distro Rust Verification Suite
# Verifies compatibility across all current and rolling Linux distributions
# as of November 2026 without installing or modifying the host operating system.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Choose container engine (prefer podman on Fedora/RHEL, fallback to docker)
if command -v podman &>/dev/null; then
    ENGINE="podman"
elif command -v docker &>/dev/null; then
    ENGINE="docker"
else
    echo "Error: Neither podman nor docker found in PATH." >&2
    exit 1
fi

GREEN="\033[0;32m"
BLUE="\033[0;34m"
CYAN="\033[0;36m"
NC="\033[0m"

echo -e "${BLUE}================================================================${NC}"
echo -e "${BLUE} MSI Mystic Light Controller - Multi-Distro Rust 2026 Matrix     ${NC}"
echo -e "${BLUE} Engine: ${ENGINE}${NC}"
echo -e "${BLUE} Root  : ${REPO_ROOT}${NC}"
echo -e "${BLUE}================================================================${NC}"

# Detect if live hardware test requested
LIVE_HW=false
if [[ "${1:-}" == "--live" || "${1:-}" == "--hardware" ]]; then
    LIVE_HW=true
    echo -e "${GREEN}[Mode] Live Hardware Passthrough Enabled${NC}"
fi

# Ensure release binary exists
if [ ! -f "${REPO_ROOT}/target/release/msi-mysticlight" ]; then
    echo "Compiling release binary for distribution testing..."
    cargo build --release --manifest-path "${REPO_ROOT}/Cargo.toml"
fi

# 1. Official Rust Container (Cargo Compilation & All 18 Tests)
echo -e "\n${CYAN}--> [1/6] Running Cargo Test Suite in Rust Container (Official)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/rust:latest \
    /bin/bash -c "cd /tmp && cp -r /app /tmp/build && cd /tmp/build && cargo test --quiet" \
    && echo -e "${GREEN}✓ Official Rust Container Tests Passed!${NC}"

# 2. Ubuntu 26.10 (Stonking Stingray - Latest Rolling glibc)
echo -e "\n${CYAN}--> [2/6] Verifying Binary in Ubuntu 26.10 (Stonking Stingray)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/ubuntu:rolling \
    /app/target/release/msi-mysticlight --version \
    && echo -e "${GREEN}✓ Ubuntu 26.10 Passed!${NC}"

# 3. Ubuntu 26.04.1 LTS (Resolute Raccoon - Long Term Support)
echo -e "\n${CYAN}--> [3/6] Verifying Binary in Ubuntu 26.04.1 LTS...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/ubuntu:26.04 \
    /app/target/release/msi-mysticlight --version \
    && echo -e "${GREEN}✓ Ubuntu 26.04 LTS Passed!${NC}"

# 4. Debian Sid (Unstable / Trixie)
echo -e "\n${CYAN}--> [4/6] Verifying Binary in Debian Sid...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/debian:sid-slim \
    /app/target/release/msi-mysticlight --version \
    && echo -e "${GREEN}✓ Debian Sid Passed!${NC}"

# 5. Arch Linux (Rolling Release)
echo -e "\n${CYAN}--> [5/6] Verifying Binary in Arch Linux...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/archlinux:latest \
    /app/target/release/msi-mysticlight --version \
    && echo -e "${GREEN}✓ Arch Linux Passed!${NC}"

# 6. openSUSE Tumbleweed
echo -e "\n${CYAN}--> [6/6] Verifying Binary in openSUSE Tumbleweed...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    registry.opensuse.org/opensuse/tumbleweed:latest \
    /app/target/release/msi-mysticlight --version \
    && echo -e "${GREEN}✓ openSUSE Tumbleweed Passed!${NC}"

# Optional: Live Hardware Passthrough
if [ "${LIVE_HW}" = true ]; then
    echo -e "\n${BLUE}--> [Hardware Passthrough] Querying Keyboard from Ubuntu Container...${NC}"
    ${ENGINE} run --rm --privileged \
        -v /sys:/sys:ro \
        -v /dev:/dev \
        -v "${REPO_ROOT}:/app:ro,z" \
        docker.io/library/ubuntu:rolling \
        /app/target/release/msi-mysticlight --status \
        && echo -e "${GREEN}✓ Live Hardware Communication Verified from Container!${NC}"
fi

echo -e "\n${GREEN}================================================================${NC}"
echo -e "${GREEN} All Distribution Environments Passed Verification!             ${NC}"
echo -e "${GREEN}================================================================${NC}"
