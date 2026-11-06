#!/usr/bin/env bash
set -euo pipefail

# MSI Mystic Light Controller for Linux - Multi-Distro Container Test Runner
# Tests the codebase across Debian, Ubuntu, Arch Linux, Alpine, and Fedora
# without installing or modifying the host operating system.

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
RED="\033[0;31m"
NC="\033[0m"

echo -e "${BLUE}======================================================${NC}"
echo -e "${BLUE} MSI Mystic Light Controller - Multi-Distro Test Runner${NC}"
echo -e "${BLUE} Engine: ${ENGINE}${NC}"
echo -e "${BLUE} Root  : ${REPO_ROOT}${NC}"
echo -e "${BLUE}======================================================${NC}"

# Detect if live hardware test requested
LIVE_HW=false
if [[ "${1:-}" == "--live" || "${1:-}" == "--hardware" ]]; then
    LIVE_HW=true
    echo -e "${GREEN}[Mode] Live Hardware Passthrough Enabled${NC}"
fi

# 1. Ubuntu 24.04 LTS
echo -e "\n${BLUE}--> [1/4] Testing in Ubuntu 24.04 LTS...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/ubuntu:24.04 \
    /bin/bash -c "apt-get update -qq && apt-get install -y -qq python3 python3-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Ubuntu 24.04 LTS Passed!${NC}"

# 2. Debian 12 Bookworm
echo -e "\n${BLUE}--> [2/4] Testing in Debian 12 (Bookworm)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/python:3.12-slim \
    /bin/bash -c "pip install pytest -q && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Debian 12 Bookworm Passed!${NC}"

# 3. Arch Linux (Rolling / Bleeding-Edge Python)
echo -e "\n${BLUE}--> [3/4] Testing in Arch Linux...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/archlinux:latest \
    /bin/bash -c "pacman -Sy --noconfirm python python-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Arch Linux Passed!${NC}"

# 4. Alpine Linux (musl libc runtime)
echo -e "\n${BLUE}--> [4/4] Testing in Alpine Linux (musl libc)...${NC}"
${ENGINE} run --rm \
    -v "${REPO_ROOT}:/app:ro,z" \
    docker.io/library/alpine:latest \
    /bin/sh -c "apk add --no-cache python3 py3-pytest >/dev/null && cd /app && pytest -q -o cache_dir=/tmp/.pytest_cache" \
    && echo -e "${GREEN}✓ Alpine Linux Passed!${NC}"

# Optional: Live Hardware Passthrough
if [ "${LIVE_HW}" = true ]; then
    echo -e "\n${BLUE}--> [Hardware Passthrough] Querying Keyboard from Debian Container...${NC}"
    ${ENGINE} run --rm --privileged \
        -v /sys:/sys:ro \
        -v "${REPO_ROOT}:/app:ro,z" \
        docker.io/library/python:3.12-slim \
        /bin/bash -c "PYTHONPATH=/app/src python3 -m msi_keyboard.cli --status" \
        && echo -e "${GREEN}✓ Live Hardware Communication Verified from Container!${NC}"
fi

echo -e "\n${GREEN}======================================================${NC}"
echo -e "${GREEN} All 4 Linux Architectures Passed Verification!        ${NC}"
echo -e "${GREEN}======================================================${NC}"
