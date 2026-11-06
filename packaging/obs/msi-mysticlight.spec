#
# spec file for package msi-mysticlight
#
# Copyright (c) 2026 Anas Kummali <anas@milkbun.in>
#
# All modifications and additions to the file contributed by third parties
# remain the property of their copyright owners, unless otherwise agreed
# upon. The license for this file, and modifications and additions to the
# file, is the same license as for the pristine package itself (unless the
# license for the pristine package is not an Open Source License, in which
# case the license is the MIT License). An "Open Source License" is a
# license that conforms to the Open Source Definition (Version 1.9)
# published by the Open Source Initiative.

Name:           msi-mysticlight
Version:        1.2.0
Release:        0
Summary:        MSI Mystic Light Controller & GUI for Linux
License:        GPL-3.0-only
Group:          Hardware/Other
URL:            https://github.com/anas-kummali/msi-mysticlight-controller-linux
Source0:        msi-mysticlight-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.70
BuildRequires:  pkgconfig
BuildRequires:  hicolor-icon-theme
BuildRequires:  update-desktop-files

%if 0%{?suse_version}
BuildRequires:  pkgconfig(udev)
BuildRequires:  pkgconfig(systemd)
%endif

%description
High-performance native Rust controller and GUI application for MSI
MysticLight 24-zone and 4-zone keyboard backlights. Integrates OpenRGB
and LightShow features into a single unified desktop experience.

%prep
%autosetup -n msi-mysticlight-%{version}

%build
cargo build --release --bin msi-mysticlight --bin msi-mysticlight-gui

%install
install -D -m 0755 target/release/msi-mysticlight %{buildroot}%{_bindir}/msi-mysticlight
install -D -m 0755 target/release/msi-mysticlight-gui %{buildroot}%{_bindir}/msi-mysticlight-gui
ln -sf msi-mysticlight %{buildroot}%{_bindir}/msi-keyboard
ln -sf msi-mysticlight %{buildroot}%{_bindir}/msi-rgb

# Udev rule
install -D -m 0644 packaging/99-msi-mysticlight.rules %{buildroot}%{_udevrulesdir}/99-msi-mysticlight.rules

# Systemd user units
install -D -m 0644 packaging/systemd/msi-keyboard.service %{buildroot}%{_userunitdir}/msi-keyboard.service
install -D -m 0644 packaging/systemd/msi-keyboard-resume.service %{buildroot}%{_userunitdir}/msi-keyboard-resume.service

# Desktop integration
install -D -m 0644 packaging/msi-mysticlight.desktop %{buildroot}%{_datadir}/applications/msi-mysticlight.desktop
install -D -m 0644 packaging/icons/msi-mysticlight.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/msi-mysticlight.svg

%if 0%{?suse_version}
%suse_update_desktop_file msi-mysticlight Settings HardwareSettings
%endif

%post
%{?udev_rules_update:%udev_rules_update}
udevadm control --reload-rules 2>/dev/null || :
udevadm trigger 2>/dev/null || :

%postun
%{?udev_rules_update:%udev_rules_update}
udevadm control --reload-rules 2>/dev/null || :
udevadm trigger 2>/dev/null || :

%files
%license LICENSE
%doc README.md
%{_bindir}/msi-mysticlight
%{_bindir}/msi-mysticlight-gui
%{_bindir}/msi-keyboard
%{_bindir}/msi-rgb
%{_udevrulesdir}/99-msi-mysticlight.rules
%{_userunitdir}/msi-keyboard.service
%{_userunitdir}/msi-keyboard-resume.service
%{_datadir}/applications/msi-mysticlight.desktop
%{_datadir}/icons/hicolor/scalable/apps/msi-mysticlight.svg

%changelog

