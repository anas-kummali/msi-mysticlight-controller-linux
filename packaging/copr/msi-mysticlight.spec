Name:           msi-mysticlight
Version:        1.2.0
Release:        1%{?dist}
Summary:        MSI Mystic Light Controller & GUI for Linux
License:        GPL-3.0-only
URL:            https://github.com/anas-kummali/msi-mysticlight-controller-linux
Source0:        msi-mysticlight-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.70
BuildRequires:  gcc
BuildRequires:  systemd-rpm-macros

%description
High-performance native Rust controller and GUI application for MSI
MysticLight 24-zone and 4-zone keyboard backlights. Integrates OpenRGB
and LightShow features into a single unified desktop experience.

%prep
%autosetup -n msi-mysticlight-%{version}

%build
cargo build --release --bin msi-mysticlight --bin msi-mysticlight-gui

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_udevrulesdir}
mkdir -p %{buildroot}%{_userunitdir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/scalable/apps

install -m 0755 target/release/msi-mysticlight %{buildroot}%{_bindir}/msi-mysticlight
install -m 0755 target/release/msi-mysticlight-gui %{buildroot}%{_bindir}/msi-mysticlight-gui
ln -sf msi-mysticlight %{buildroot}%{_bindir}/msi-keyboard
ln -sf msi-mysticlight %{buildroot}%{_bindir}/msi-rgb

install -m 0644 packaging/99-msi-mysticlight.rules %{buildroot}%{_udevrulesdir}/99-msi-mysticlight.rules
install -m 0644 packaging/systemd/msi-keyboard.service %{buildroot}%{_userunitdir}/msi-keyboard.service
install -m 0644 packaging/systemd/msi-keyboard-resume.service %{buildroot}%{_userunitdir}/msi-keyboard-resume.service
install -m 0644 packaging/msi-mysticlight.desktop %{buildroot}%{_datadir}/applications/msi-mysticlight.desktop
install -m 0644 packaging/icons/msi-mysticlight.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/msi-mysticlight.svg

%post
udevadm control --reload-rules 2>/dev/null || :
udevadm trigger 2>/dev/null || :

%postun
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
* Sat Nov 07 2026 Anas Kummali <anas@milkbun.in> - 1.2.0-1
- Release v1.2.0: Added Studio GUI, 16 presets, custom gradients, diagnostics, systemd integration
