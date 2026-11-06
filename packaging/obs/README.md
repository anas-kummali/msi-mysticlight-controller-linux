# openSUSE Build Service (OBS) Packaging

openSUSE packages are hosted on the **Open Build Service (OBS)** at [build.opensuse.org](https://build.opensuse.org/). OBS automatically compiles packages and generates native Zypper repositories for:
- **openSUSE Tumbleweed** (Rolling Release)
- **openSUSE Leap 15.6 / Leap 16.0**
- **SUSE Linux Enterprise Server (SLES)**

---

## User Installation on openSUSE

Once published on OBS under your home project (e.g., `home:anas-kummali`):

### openSUSE Tumbleweed
```bash
sudo zypper addrepo https://download.opensuse.org/repositories/home:anas-kummali/openSUSE_Tumbleweed/home:anas-kummali.repo
sudo zypper refresh
sudo zypper install -y msi-mysticlight
```

### openSUSE Leap 15.6
```bash
sudo zypper addrepo https://download.opensuse.org/repositories/home:anas-kummali/15.6/home:anas-kummali.repo
sudo zypper refresh
sudo zypper install -y msi-mysticlight
```

### 1-Click Install (YaST)
Users can also visit `https://software.opensuse.org/package/msi-mysticlight` and click **"Direct Install / 1-Click"** to launch YaST Software Manager.

---

## Maintainer Guide: Publishing to OBS

### Method 1: Using the Web Interface (Simplest)
1. Register/Login at [build.opensuse.org](https://build.opensuse.org/).
2. Navigate to your Home Project (`home:<username>`).
3. Click **"Create Package"**:
   - **Name:** `msi-mysticlight`
   - **Title:** `MSI Mystic Light Controller & GUI for Linux`
4. Under the new package, upload:
   - `packaging/obs/msi-mysticlight.spec`
   - `packaging/obs/msi-mysticlight.changes`
   - `target/packages/msi-mysticlight-1.2.0.tar.gz` (or `_service`)
5. Enable target repositories:
   - `openSUSE:Factory` (Tumbleweed)
   - `openSUSE:Leap:15.6`
6. OBS will immediately queue the build, compile the package in clean sandboxes, and publish Zypper repository metadata.

---

### Method 2: Using the Command-Line Tool (`osc`)
```bash
# Install Open Build Service client
sudo zypper install osc # (on openSUSE) or sudo dnf install osc (on Fedora)

# Checkout your home project
osc checkout home:anas-kummali
cd home:anas-kummali

# Create package directory
osc mkpac msi-mysticlight
cd msi-mysticlight

# Copy packaging files
cp /path/to/packaging/obs/* .
cp /path/to/target/packages/msi-mysticlight-1.2.0.tar.gz .

# Add files and commit
osc add *
osc commit -m "Initial import of msi-mysticlight v1.2.0"
```
