Name:           phonon
Version:        0.1.0
Release:        1%{?dist}
Summary:        Phonon Universal Multi-Scale Visual Studio & Quantum CAD Simulation Engine
License:        MIT OR Apache-2.0
URL:            https://phonon.aerovex.net
Source0:        phonon-v%{version}-x86_64-unknown-linux-gnu.tar.gz

BuildArch:      x86_64
Requires:       glibc >= 2.31

%description
Phonon is a multi-scale, multi-physics semiconductor and quantum acoustic
simulation platform. It provides a headless CLI simulation engine and a native
hardware-accelerated visual studio CAD interface for quantum acoustic metamaterials,
superconducting artificial atoms, and cryogenic electro-thermal devices.

%prep
%setup -q -c

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir} \
         %{buildroot}%{_datadir}/applications \
         %{buildroot}%{_datadir}/icons/hicolor/scalable/apps \
         %{buildroot}%{_datadir}/bash-completion/completions \
         %{buildroot}%{_datadir}/zsh/site-functions

# Install binary
install -m 0755 bin/phonon %{buildroot}%{_bindir}/phonon

# Install desktop file and icon
install -m 0644 share/applications/phonon.desktop %{buildroot}%{_datadir}/applications/phonon.desktop
install -m 0644 share/icons/hicolor/scalable/apps/phonon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/phonon.svg

# Install autocompletions
install -m 0644 share/completions/phonon.bash %{buildroot}%{_datadir}/bash-completion/completions/phonon
install -m 0644 share/completions/_phonon.zsh %{buildroot}%{_datadir}/zsh/site-functions/_phonon

%post
if [ -x %{_bindir}/update-desktop-database ]; then
    %{_bindir}/update-desktop-database &> /dev/null || :
fi

%postun
if [ -x %{_bindir}/update-desktop-database ]; then
    %{_bindir}/update-desktop-database &> /dev/null || :
fi

%files
%{_bindir}/phonon
%{_datadir}/applications/phonon.desktop
%{_datadir}/icons/hicolor/scalable/apps/phonon.svg
%{_datadir}/bash-completion/completions/phonon
%{_datadir}/zsh/site-functions/_phonon
%license LICENSE
%doc README.md

%changelog
* Wed Oct 01 2026 Aerovex Technologies <engineering@aerovex.net> - 0.1.0-1
- Initial production release v0.1.0 with unified CLI and Desktop Studio
