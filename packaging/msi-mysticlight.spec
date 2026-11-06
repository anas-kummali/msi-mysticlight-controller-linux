Name:           msi-mysticlight
Version:        1.1.0
Release:        1%{?dist}
Summary:        MSI Mystic Light Controller & GUI for Linux
License:        GPL-3.0-only
URL:            https://github.com/anas-kummali/msi-mysticlight-controller-linux

BuildArch:      x86_64

%description
High-performance native Rust controller and GUI application for MSI
MysticLight 24-zone and 4-zone keyboard backlights. Integrates OpenRGB
and LightShow features into a single unified desktop experience.

%prep

%build

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_udevrulesdir}
mkdir -p %{buildroot}%{_userunitdir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/scalable/apps

install -m 0755 %{_sourcedir}/msi-mysticlight %{buildroot}%{_bindir}/msi-mysticlight
install -m 0755 %{_sourcedir}/msi-mysticlight-gui %{buildroot}%{_bindir}/msi-mysticlight-gui
ln -sf msi-mysticlight %{buildroot}%{_bindir}/msi-keyboard
ln -sf msi-mysticlight %{buildroot}%{_bindir}/msi-rgb

install -m 0644 %{_sourcedir}/99-msi-mysticlight.rules %{buildroot}%{_udevrulesdir}/99-msi-mysticlight.rules
install -m 0644 %{_sourcedir}/msi-keyboard.service %{buildroot}%{_userunitdir}/msi-keyboard.service
install -m 0644 %{_sourcedir}/msi-keyboard-resume.service %{buildroot}%{_userunitdir}/msi-keyboard-resume.service
install -m 0644 %{_sourcedir}/msi-mysticlight.desktop %{buildroot}%{_datadir}/applications/msi-mysticlight.desktop
install -m 0644 %{_sourcedir}/msi-mysticlight.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/msi-mysticlight.svg

%post
udevadm control --reload-rules 2>/dev/null || :
udevadm trigger 2>/dev/null || :

%postun
udevadm control --reload-rules 2>/dev/null || :
udevadm trigger 2>/dev/null || :

%files
%{_bindir}/msi-mysticlight
%{_bindir}/msi-mysticlight-gui
%{_bindir}/msi-keyboard
%{_bindir}/msi-rgb
%{_udevrulesdir}/99-msi-mysticlight.rules
%{_userunitdir}/msi-keyboard.service
%{_userunitdir}/msi-keyboard-resume.service
%{_datadir}/applications/msi-mysticlight.desktop
%{_datadir}/icons/hicolor/scalable/apps/msi-mysticlight.svg
