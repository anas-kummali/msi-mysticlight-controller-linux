# Fedora COPR Repository Setup

Fedora COPR (Cool Other Package Repo) provides automated builds and binary RPM repositories for Fedora Linux (41, 42, 43, 44, 45, Rawhide) and EPEL/CentOS Stream.

---

## User Installation (Fedora / RHEL / CentOS)

Once the COPR repo is published:

```bash
# 1. Enable the COPR repository
sudo dnf copr enable anas-kummali/msi-mysticlight

# 2. Install the package
sudo dnf install -y msi-mysticlight

# 3. Reload udev rules & launch GUI
sudo udevadm control --reload-rules && sudo udevadm trigger
msi-mysticlight-gui
```

---

## Maintainer: Setting Up COPR Project

1. Log in to [Fedora COPR](https://copr.fedorainfracloud.org/) using your Fedora Account (FAS) or GitHub login.
2. Click **New Project**:
   - **Project Name:** `msi-mysticlight`
   - **Description:** MSI Mystic Light Controller & GUI for Linux (24-Zone & 4-Zone Keyboards)
   - **Instructions:** `sudo dnf copr enable anas-kummali/msi-mysticlight && sudo dnf install msi-mysticlight`
   - Select target chroots (e.g. `fedora-43-x86_64`, `fedora-44-x86_64`, `fedora-45-x86_64`, `fedora-rawhide-x86_64`).

### Method A: Automated Git Webhook (Recommended)
In the COPR project settings under **Packages**:
- **Source Type:** SCM (Git)
- **Clone URL:** `https://github.com/anas-kummali/msi-mysticlight-controller-linux.git`
- **Committish:** `main`
- **Spec file:** `packaging/copr/msi-mysticlight.spec`
- COPR will automatically detect `.copr/Makefile` and build new packages on every git push or webhook trigger!

### Method B: Upload Pre-Built Source RPM
Run the local generator script:
```bash
./packaging/build_srpm.sh
```
This produces `target/copr/msi-mysticlight-1.2.0-1.fc45.src.rpm`.
Upload this file under **New Build** -> **Upload SRPM** in COPR or run:
```bash
copr-cli build anas-kummali/msi-mysticlight target/copr/*.src.rpm
```
