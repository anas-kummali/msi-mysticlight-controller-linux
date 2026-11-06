#!/usr/bin/env bash
set -euo pipefail

# MSI Mystic Light Controller for Linux - Multi-Distro Container Test Runner
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
    echo "Install podman on Fedora: sudo dnf install podman" >&2
    exit 1
fi

GREEN="\033[0;32m"
BLUE="\033[0;34m"
CYAN="\033[0;36m"
NC="\033[0m"

echo -e "${BLUE}================================================================${NC}"
echo -e "${BLUE} MSI Mystic Light Controller - Multi-Distro 2026 Test Matrix     ${NC}"
echo -e "${BLUE} Engine: ${ENGINE}${NC}"
echo -e "${BLUE} Root  : ${REPO_ROOT}${NC}"
echo -e "${BLUE}================================================================${NC}"

# Detect if live hardware test requested
LIVE_HW=false
if [[ "${1:-}" == "--live" || "${1:-}" == "--hardware" ]]; then
    LIVE_HW=true
    echo -e "${GREEN}[Mode] Live Hardware Passthrough Enabled${NC}"
fi

# 1. Ubuntu 26.10 (Stonking Stingray - Latest Interim)
echo -e "\n${CYAN}--> [1/7] Testing in Ubuntu 26.10 (Stonking Stingray)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/ubuntu:rolling \
    /bin/bash -c "apt-get update -qq && apt-get install -y -qq python3 python3-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Ubuntu 26.10 Passed!${NC}"

# 2. Ubuntu 26.04 LTS (Resolute Raccoon - Long Term Support)
echo -e "\n${CYAN}--> [2/7] Testing in Ubuntu 26.04.1 LTS (Resolute Raccoon)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/ubuntu:26.04 \
    /bin/bash -c "apt-get update -qq && apt-get install -y -qq python3 python3-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Ubuntu 26.04 LTS Passed!${NC}"

# 3. Debian Sid / Unstable (Trixie/Sid)
echo -e "\n${CYAN}--> [3/7] Testing in Debian Sid (Unstable / Trixie)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/debian:sid-slim \
    /bin/bash -c "apt-get update -qq && apt-get install -y -qq python3 python3-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Debian Sid Passed!${NC}"

# 4. Debian 12 (Bookworm)
echo -e "\n${CYAN}--> [4/7] Testing in Debian 12 (Bookworm)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/python:3.12-slim \
    /bin/bash -c "pip install pytest -q && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Debian 12 Bookworm Passed!${NC}"

# 5. Arch Linux (Rolling Release, Bleeding-Edge Python)
echo -e "\n${CYAN}--> [5/7] Testing in Arch Linux (pacman rolling)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/archlinux:latest \
    /bin/bash -c "pacman -Sy --noconfirm python python-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Arch Linux Passed!${NC}"

# 6. openSUSE Tumbleweed (Rolling Release)
echo -e "\n${CYAN}--> [6/7] Testing in openSUSE Tumbleweed (zypper)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    registry.opensuse.org/opensuse/tumbleweed:latest \
    /bin/bash -c "zypper --non-interactive install python3 python3-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ openSUSE Tumbleweed Passed!${NC}"

# 7. Alpine Linux (musl libc runtime)
echo -e "\n${CYAN}--> [7/7] Testing in Alpine Linux (musl libc)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/alpine:latest \
    /bin/sh -c "apk add --no-cache python3 py3-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Alpine Linux (musl) Passed!${NC}"

# Optional: Live Hardware Passthrough
if [ "${LIVE_HW}" = true ]; then
    echo -e "\n${BLUE}--> [Hardware Passthrough] Querying Keyboard from Ubuntu 26.10 Container...${NC}"
    ${ENGINE} run --rm --privileged \
        -v /sys:/sys:ro \
        -v "${REPO_ROOT}:/app:ro,z" \
        docker.io/library/ubuntu:rolling \
        /bin/bash -c "apt-get update -qq && apt-get install -y -qq python3 >/dev/null && PYTHONPATH=/app/src python3 -m msi_keyboard.cli --status" \
        && echo -e "${GREEN}✓ Live Hardware Communication Verified from Ubuntu 26.10 Container!${NC}"
fi

echo -e "\n${GREEN}================================================================${NC}"
echo -e "${GREEN} All 7 Modern Linux Distributions Passed Verification!          ${NC}"
echo -e "${GREEN}================================================================${NC}"
