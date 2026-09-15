# Release binaries are built with `strip = "symbols"` (see [profile.release]
# in Cargo.toml) across every packaging channel (deb, AUR, GitHub releases),
# so there is no debug info here for Fedora's debuginfo/debugsource split to
# extract in the first place.
%global debug_package %{nil}

Name:           rustmius
Version:        2.5.0
Release:        4%{?dist}
Summary:        Local Termius alternative for Linux (GTK4)

License:        AGPL-3.0-or-later
URL:            https://github.com/Cleboost/Rustmius
# Generated locally from the working tree, see packages/rpm/build-rpm.sh
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  desktop-file-utils
BuildRequires:  pkgconfig(gtk4) >= 4.12
BuildRequires:  pkgconfig(vte-2.91-gtk4)
BuildRequires:  pkgconfig(libssh2)
BuildRequires:  pkgconfig(openssl)
BuildRequires:  pkgconfig(zlib)

# Runtime deps are also picked up automatically from the linked binary
# (soname-based Requires), these just make the intent explicit.
Requires:       gtk4%{?_isa} >= 4.12
Requires:       vte291-gtk4%{?_isa}
Requires:       hicolor-icon-theme

%description
Rustmius is a modern, fast, and fully local alternative to Termius,
built with Rust and GTK4. It provides an integrated SSH terminal
(via VTE), an advanced SFTP explorer with drag & drop, a host
manager, and secure secret storage through the system keyring.

%prep
%autosetup -n %{name}-%{version}

%build
if [ "${SKIP_BUILD:-0}" = "1" ] && [ -x target/release/%{name} ]; then
    echo ">> Reusing prebuilt %{name} binary (SKIP_BUILD=1)"
else
    cargo build --release --locked
fi

%install
install -Dm0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

install -Dm0644 packages/org.rustmius.Rustmius.desktop \
    %{buildroot}%{_datadir}/applications/org.rustmius.Rustmius.desktop

install -dm0755 %{buildroot}%{_datadir}/icons/hicolor/512x512/apps
if command -v magick >/dev/null 2>&1; then
    magick packages/rustmius.png -resize 512x512 \
        %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/%{name}.png
elif command -v convert >/dev/null 2>&1; then
    convert packages/rustmius.png -resize 512x512 \
        %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/%{name}.png
else
    install -m0644 packages/rustmius.png \
        %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/%{name}.png
fi

install -dm0755 %{buildroot}%{_mandir}/man1
sed "s/@VERSION@/%{version}/g" packages/deb/rustmius.1 \
    > %{buildroot}%{_mandir}/man1/%{name}.1

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/org.rustmius.Rustmius.desktop

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_datadir}/applications/org.rustmius.Rustmius.desktop
%{_datadir}/icons/hicolor/512x512/apps/%{name}.png
%{_mandir}/man1/%{name}.1*

%changelog
* Wed Aug 12 2026 Subhan Gadirli <subhanqedirli@protonmail.com> - 2.5.0-4
- Support SKIP_BUILD=1 to reuse a prebuilt binary (used by CI to avoid
  recompiling for each package format).

* Wed Aug 12 2026 Subhan Gadirli <subhanqedirli@protonmail.com> - 2.5.0-3
- Fix dark-mode sync: xdg-desktop-portal-gnome double-wraps the
  Settings.Read/SettingChanged value in an extra GVariant "v" layer,
  which needs peeling before reading the color-scheme uint32.

* Wed Aug 12 2026 Subhan Gadirli <subhanqedirli@protonmail.com> - 2.5.0-2
- Sync app light/dark mode with the system preference via the XDG
  Desktop Portal (plain GTK4 does not do this on its own).

* Wed Aug 12 2026 Subhan Gadirli <subhanqedirli@protonmail.com> - 2.5.0-1
- Initial RPM packaging for Fedora.
