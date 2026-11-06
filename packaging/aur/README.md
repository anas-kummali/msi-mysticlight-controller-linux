# Arch User Repository (AUR) Packages

This directory contains official PKGBUILD definitions for Arch Linux, Manjaro, EndeavourOS, and SteamOS:

1. **`msi-mysticlight-bin`** (Recommended for end-users):
   - Fast 5-second installation.
   - Repacks official signed pre-compiled release binaries with desktop integration, systemd user services, and udev hardware permissions.
2. **`msi-mysticlight`** (Source build):
   - Builds natively from source code using `cargo`.

---

## Installation via AUR Helpers

### Using `yay`
```bash
# Recommended binary package
yay -S msi-mysticlight-bin

# Or build from source
yay -S msi-mysticlight
```

### Using `paru`
```bash
paru -S msi-mysticlight-bin
```

---

## Manual Installation without Helper

```bash
git clone https://aur.archlinux.org/msi-mysticlight-bin.git
cd msi-mysticlight-bin
makepkg -si
```

---

## Maintainer Guide: Pushing to AUR

To publish or update the packages on `aur.archlinux.org`:

```bash
# 1. Clone your AUR repository
git clone ssh://aur@aur.archlinux.org/msi-mysticlight-bin.git aur-bin
cd aur-bin

# 2. Copy the PKGBUILD and .SRCINFO
cp /path/to/msi-mysticlight-controller-linux/packaging/aur/msi-mysticlight-bin/* .

# 3. Commit and push
git add PKGBUILD .SRCINFO
git commit -m "Update to v1.2.0"
git push origin master
```

