# Launchpad PPA (Ubuntu, Debian, Pop!_OS, Linux Mint)

Ubuntu and Debian derivatives support automated personal package archives hosted on Canonical's **Launchpad PPA**.

---

## User Installation (Ubuntu / Linux Mint / Pop!_OS)

```bash
# 1. Add PPA repository
sudo add-apt-repository -y ppa:anas-kummali/msi-mysticlight

# 2. Update and install
sudo apt-get update
sudo apt-get install -y msi-mysticlight

# 3. Reload udev and start GUI
sudo udevadm control --reload-rules && sudo udevadm trigger
msi-mysticlight-gui
```

---

## Maintainer Guide: Setting Up Launchpad PPA

1. **Activate PPA:**
   - Log in to [Launchpad](https://launchpad.net/).
   - Go to `https://launchpad.net/~anas-kummali/+activate-ppa`.
   - Name: `msi-mysticlight`
   - Description: `MSI Mystic Light Controller & GUI for Linux`
2. **GPG Signing Key:**
   - Ensure your GPG public key is uploaded to Ubuntu keyserver (`keyserver.ubuntu.com`) and linked to your Launchpad profile.
3. **Build & Upload Source Package:**
   ```bash
   ./packaging/build_ppa.sh
   dput ppa:anas-kummali/msi-mysticlight ../msi-mysticlight_*_source.changes
   ```
4. Canonical's Launchpad build farm will automatically compile native `.deb` binaries for `noble` (24.04 LTS), `jammy` (22.04 LTS), and `oracular` (24.10) for both `amd64` and `arm64`.
