#!/usr/bin/env python3
import os
import re
import sys
import json
import urllib.request
import urllib.error

CRED_PATH = os.path.expanduser("~/.git-credentials")
if not os.path.exists(CRED_PATH):
    print("Error: ~/.git-credentials not found", file=sys.stderr)
    sys.exit(1)

with open(CRED_PATH, "r", encoding="utf-8") as f:
    cred_text = f.read()

m = re.search(r"https://([^:]+):([^@]+)@github\.com", cred_text)
if not m:
    print("Error: GitHub token not found in ~/.git-credentials", file=sys.stderr)
    sys.exit(1)

TOKEN = m.group(2)
REPO = "anas-kummali/msi-mysticlight-controller-linux"
BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PACKAGES_DIR = os.path.join(BASE_DIR, "target", "packages")

def gh_api_request(url, data=None, method="GET", content_type="application/json"):
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", f"Bearer {TOKEN}")
    req.add_header("Accept", "application/vnd.github+json")
    req.add_header("User-Agent", "msi-mysticlight-publisher")
    if data and content_type:
        req.add_header("Content-Type", content_type)
    return req

def get_existing_releases():
    url = f"https://api.github.com/repos/{REPO}/releases"
    req = gh_api_request(url)
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode("utf-8"))

def create_or_get_release(tag, name, body, prerelease=False):
    existing = get_existing_releases()
    for rel in existing:
        if rel.get("tag_name") == tag:
            print(f"Release for {tag} already exists (ID: {rel.get('id')}).")
            return rel

    url = f"https://api.github.com/repos/{REPO}/releases"
    payload = json.dumps({
        "tag_name": tag,
        "name": name,
        "body": body,
        "draft": False,
        "prerelease": prerelease
    }).encode("utf-8")

    req = gh_api_request(url, data=payload, method="POST")
    with urllib.request.urlopen(req) as resp:
        rel = json.loads(resp.read().decode("utf-8"))
        print(f"Created release {tag} -> {rel.get('html_url')}")
        return rel

def upload_asset_to_release(release, file_path):
    filename = os.path.basename(file_path)
    if not os.path.exists(file_path):
        print(f"Warning: File {file_path} does not exist, skipping.")
        return

    # Check if already uploaded
    existing_assets = release.get("assets", [])
    for a in existing_assets:
        if a.get("name") == filename:
            print(f"  Asset {filename} already uploaded (ID: {a.get('id')}).")
            return

    upload_url_template = release.get("upload_url", "")
    base_upload_url = upload_url_template.split("{")[0]
    upload_url = f"{base_upload_url}?name={filename}"

    content_type = "application/octet-stream"
    if filename.endswith(".deb"):
        content_type = "application/vnd.debian.binary-package"
    elif filename.endswith(".rpm"):
        content_type = "application/x-rpm"
    elif filename.endswith(".tar.gz"):
        content_type = "application/gzip"

    print(f"  Uploading {filename} ({os.path.getsize(file_path) / 1024 / 1024:.1f} MB)...")
    with open(file_path, "rb") as f:
        file_bytes = f.read()

    req = gh_api_request(upload_url, data=file_bytes, method="POST", content_type=content_type)
    with urllib.request.urlopen(req) as resp:
        res = json.loads(resp.read().decode("utf-8"))
        print(f"    ✔ Successfully uploaded {filename} -> {res.get('browser_download_url')}")

def main():
    print(f"=== Publishing Releases for {REPO} ===")

    # 1. Release v1.1.0
    body_1_1_0 = """## ⚡ MSI Mystic Light Controller for Linux v1.1.0

Initial native Rust graphical user interface (GUI), curated theme engine, and multi-distribution packaging.

### ✨ Highlights
* **Native Rust GUI:** `msi-mysticlight-gui` / `msi-mysticlight --gui` built with `egui` and `eframe`.
* **24-Zone Addressing:** Eliminates the Linux 3-row limitation; controls all 6 physical rows.
* **9 Curated Themes:** Tokyo Night, Cyberpunk Neon, Synthwave, Matrix Code, Arctic Frost, Nord Aurora, Tactical Crimson, Sovereign White, Warm Amber.
* **Distribution Packaging:** Automated packages for Debian/Ubuntu (.deb), Fedora/RHEL (.rpm), and Generic Linux (.tar.gz).
* **Desktop Integration:** Desktop launcher and scalable vector icon.

### 🔒 SHA-256 Checksums
```text
121cb5106f998350f4e7ec9ade59bb7b5462f3d112f7b883f1803ee4ca57558d  msi-mysticlight-1.1.0-1.fc45.x86_64.rpm
35f2d3129e8921b19ab0cfd9ea35ffbeb5fc2dfd07e641380816f412e3c6ba2f  msi-mysticlight_1.1.0_amd64.deb
d0a86ff39af0a0d280d24bb6089432d568d6a39745503e7fdcfbfefe2fc28dc7  msi-mysticlight-1.1.0-x86_64-linux.tar.gz
```
"""

    rel_1_1_0 = create_or_get_release(
        "v1.1.0",
        "MSI Mystic Light v1.1.0 - Initial Native Rust GUI & Distribution Packaging",
        body_1_1_0
    )
    upload_asset_to_release(rel_1_1_0, os.path.join(PACKAGES_DIR, "msi-mysticlight-1.1.0-1.fc45.x86_64.rpm"))
    upload_asset_to_release(rel_1_1_0, os.path.join(PACKAGES_DIR, "msi-mysticlight_1.1.0_amd64.deb"))
    upload_asset_to_release(rel_1_1_0, os.path.join(PACKAGES_DIR, "msi-mysticlight-1.1.0-x86_64-linux.tar.gz"))

    # 2. Release v1.2.0 (Latest)
    body_1_2_0 = """## ⚡ MSI Mystic Light Controller for Linux v1.2.0

Major UI/UX modernization overhaul, spatial gaming clusters, mathematical 2D gradient studio, persistent custom profiles, and laptop power telemetry.

### ✨ What's New in v1.2.0

* **Authentic Spatial Keyboard Matrix Canvas:**
  * Physical keycaps with realistic laptop key proportions (`SPACEBAR`, `ENTER`, `SHIFT`, `BACKSPACE`).
  * Real-time LED underglow halos and active RGB top-edge color bars.
  * Monospace zone tags (`Z00`–`Z23`) and physical key legends (`ESC`, `QWERTY`, etc.).
  * Interactive hover tooltips displaying key name, zone ID, physical row, and live RGB Hex code.

* **Spatial Gaming Clusters & Multi-Zone Targeting:**
  * 1-Click targeting of essential gaming clusters:
    * `WASD Cluster` (Zones 8 & 12)
    * `Arrow Keys` (Zones 19 & 23)
    * `Numpad Cluster` (Zones 11, 15, 19, 23)
    * `QWERTY Alphas` (Zones 8..10, 12..14, 16..18)
    * `Function Row (F1–F12)` (Zones 0..3)
    * `Number Row` (Zones 4..7)
    * `Space & Modifiers` (Zones 20..22)
    * `Invert Selection` and `Clear Selection`
  * Shift-click / Ctrl-click multi-zone toggling to batch-apply colors across arbitrary zones.

* **Gradient Studio:**
  * Smooth 2D mathematical spatial color interpolation across all 24 zones.
  * Start and End color pickers with quick preset chips and swap button.
  * 4 Spatial Directions: `Horizontal`, `Vertical`, `Diagonal`, `Radial`.
  * Live computed matrix preview ribbon.

* **Custom User Profiles Library:**
  * Save custom color arrangements under custom names to `~/.config/msi-mysticlight/profiles.json`.
  * Visual list of saved profiles with 6-row chromatic ribbons, 1-click apply, and delete controls.

* **Laptop Hardware Power Telemetry:**
  * Real-time AC line connection (`🔌 AC Online` vs `🔋 Battery %`) and battery charge state directly from kernel `/sys/class/power_supply/`.
  * Integrated battery conservation tips.

* **Ergonomic Window Architecture:**
  * Top navigation bar with 5 pill tabs: `Studio Controls`, `Gradient Studio`, `Effects Gallery`, `Themes & Profiles`, `Diagnostics & Power`.
  * Persistent bottom master action deck with master brightness slider [0..10], quick power buttons (`Solid White`, `Rainbow Wave`, `Power Off`), and live hardware status toast.
  * 60 FPS real-time on-screen animation mirroring for Wave, Cycle, and Breathing.

---

### 📦 Installation Packages

| Distribution / OS | Package File | Size |
| :--- | :--- | :--- |
| **Fedora / RHEL / openSUSE** | `msi-mysticlight-1.2.0-1.fc45.x86_64.rpm` | 7.3 MB |
| **Debian / Ubuntu / Linux Mint** | `msi-mysticlight_1.2.0_amd64.deb` | 9.4 MB |
| **Generic Linux (Any distro)** | `msi-mysticlight-1.2.0-x86_64-linux.tar.gz` | 9.4 MB |

#### Quick Install (Fedora / RHEL):
```bash
sudo dnf install ./msi-mysticlight-1.2.0-1.fc45.x86_64.rpm
```

#### Quick Install (Debian / Ubuntu):
```bash
sudo apt install ./msi-mysticlight_1.2.0_amd64.deb
```

#### Quick Install (Standalone Tarball):
```bash
tar -xzf msi-mysticlight-1.2.0-x86_64-linux.tar.gz
cd msi-mysticlight-1.2.0-x86_64-linux
./install.sh
```

---

### 🔒 SHA-256 Checksums
```text
043db6f4663b4d4234f7e89875954f23dd6232f188cbff03cc7e13246c8ceb54  msi-mysticlight-1.2.0-1.fc45.x86_64.rpm
b746edab22cbc27cf4e4475da418ae68be6c13d2a6452beca3a26db7072d57a7  msi-mysticlight_1.2.0_amd64.deb
721dca7447ebeef66c51eb3575d45ef0f34e5f6f1d427814e8c8c8bb978760ce  msi-mysticlight-1.2.0-x86_64-linux.tar.gz
```
"""

    rel_1_2_0 = create_or_get_release(
        "v1.2.0",
        "MSI Mystic Light v1.2.0 - Modern UI/UX Overhaul, Gradient Studio & Custom Profiles",
        body_1_2_0
    )
    upload_asset_to_release(rel_1_2_0, os.path.join(PACKAGES_DIR, "msi-mysticlight-1.2.0-1.fc45.x86_64.rpm"))
    upload_asset_to_release(rel_1_2_0, os.path.join(PACKAGES_DIR, "msi-mysticlight_1.2.0_amd64.deb"))
    upload_asset_to_release(rel_1_2_0, os.path.join(PACKAGES_DIR, "msi-mysticlight-1.2.0-x86_64-linux.tar.gz"))

    print("\n🎉 ALL RELEASES AND ASSETS SUCCESSFULLY PUBLISHED TO GITHUB!")

if __name__ == "__main__":
    main()

