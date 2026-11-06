"""Integration test suite for the compiled Rust native binary."""

import os
import subprocess
import shutil
import pytest

BINARY_PATH = os.path.normpath(
    os.path.join(
        os.path.dirname(__file__),
        "..",
        "target",
        "release",
        "msi-mysticlight",
    )
)

CARGO_AVAILABLE = shutil.which("cargo") is not None


def is_binary_runnable() -> bool:
    """Check if the binary exists and can be executed under the current libc/kernel."""
    if not os.path.isfile(BINARY_PATH):
        return False
    try:
        res = subprocess.run([BINARY_PATH, "--version"], capture_output=True, text=True)
        return res.returncode == 0
    except (OSError, subprocess.SubprocessError):
        # e.g. FileNotFoundError on musl-based Alpine when glibc binary interpreter is absent
        return False


@pytest.fixture(scope="session", autouse=True)
def ensure_rust_binary():
    """Ensure release binary is compiled if cargo is installed."""
    if CARGO_AVAILABLE and not os.path.isfile(BINARY_PATH):
        root = os.path.normpath(os.path.join(os.path.dirname(__file__), ".."))
        subprocess.run(["cargo", "build", "--release", "--manifest-path", os.path.join(root, "Cargo.toml")], check=True)


@pytest.mark.skipif(not is_binary_runnable(), reason="Rust binary not compiled or incompatible ABI")
def test_rust_binary_help():
    res = subprocess.run([BINARY_PATH, "--help"], capture_output=True, text=True)
    assert res.returncode == 0
    assert "MSI Mystic Light Controller for Linux" in res.stdout
    assert "-c, --color" in res.stdout
    assert "-B, --brightness" in res.stdout
    assert "-w, --wave" in res.stdout
    assert "-b, --breathe" in res.stdout
    assert "-s, --cycle" in res.stdout
    assert "-r, --row" in res.stdout
    assert "-z, --zone" in res.stdout
    assert "--status" in res.stdout


@pytest.mark.skipif(not is_binary_runnable(), reason="Rust binary not compiled or incompatible ABI")
def test_rust_binary_version():
    res = subprocess.run([BINARY_PATH, "--version"], capture_output=True, text=True)
    assert res.returncode == 0
    assert "msi-mysticlight 1.0.0" in res.stdout.strip()


@pytest.mark.skipif(not is_binary_runnable(), reason="Rust binary not compiled or incompatible ABI")
def test_rust_binary_status():
    res = subprocess.run([BINARY_PATH, "--status"], capture_output=True, text=True)
    assert res.returncode == 0
    assert "MSI Mystic Light Controller for Linux" in res.stdout
    assert "Total Zones     : 24 Zones" in res.stdout
    assert "Total Rows      : 6 Physical Rows" in res.stdout


@pytest.mark.skipif(not is_binary_runnable(), reason="Rust binary not compiled or incompatible ABI")
def test_rust_binary_invalid_color():
    res = subprocess.run([BINARY_PATH, "-c", "invalid_color_123"], capture_output=True, text=True)
    assert res.returncode == 1
    assert "Invalid color" in res.stderr


@pytest.mark.skipif(not is_binary_runnable(), reason="Rust binary not compiled or incompatible ABI")
def test_rust_binary_invalid_brightness():
    res = subprocess.run([BINARY_PATH, "-B", "99"], capture_output=True, text=True)
    assert res.returncode == 1
    assert "out of range" in res.stderr


@pytest.mark.skipif(not is_binary_runnable(), reason="Rust binary not compiled or incompatible ABI")
def test_rust_binary_invalid_row():
    res = subprocess.run([BINARY_PATH, "-r", "7"], capture_output=True, text=True)
    assert res.returncode == 1
    assert "out of range" in res.stderr


@pytest.mark.skipif(not is_binary_runnable(), reason="Rust binary not compiled or incompatible ABI")
def test_rust_binary_invalid_zone():
    res = subprocess.run([BINARY_PATH, "-z", "30"], capture_output=True, text=True)
    assert res.returncode == 1
    assert "out of range" in res.stderr
