# Rust crates come from a vendored tarball, so the build runs offline.
%global debug_package %{nil}

Name:           whatsup-desktop
Version:        1.0.0
Release:        1%{?dist}
Summary:        WhatsApp Web for GNOME
License:        GPL-3.0-or-later
URL:            https://github.com/inittux111/whatsup-desktop
Source0:        %{name}-%{version}.tar.gz
Source1:        %{name}-%{version}-vendor.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  pkgconfig(libadwaita-1)
BuildRequires:  pkgconfig(webkitgtk-6.0)
BuildRequires:  desktop-file-utils
BuildRequires:  make

# Fedora and openSUSE name a few packages differently.
%if 0%{?suse_version}
BuildRequires:  appstream-glib
Requires:       libadwaita-1-0
Requires:       libwebkitgtk-6_0-4
Requires:       google-noto-coloremoji-fonts
%else
BuildRequires:  libappstream-glib
Requires:       libadwaita
Requires:       webkitgtk6.0
Requires:       google-noto-color-emoji-fonts
%endif

%description
WhatsUp runs WhatsApp Web in a native GNOME window that stays responsive,
follows your theme and keeps your account data private.

%prep
%autosetup -n %{name}-%{version}
tar -xzf %{SOURCE1}
mkdir -p .cargo
cat > .cargo/config.toml <<'CONF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
CONF

%build
cargo build --release --offline --locked

%check
cargo test --release --offline --locked
desktop-file-validate data/app.whatsup.WhatsUp.desktop
appstream-util validate-relax --nonet data/app.whatsup.WhatsUp.metainfo.xml

%install
make install DESTDIR=%{buildroot} PREFIX=%{_prefix}

%files
%{_bindir}/%{name}
%{_datadir}/applications/app.whatsup.WhatsUp.desktop
%{_datadir}/metainfo/app.whatsup.WhatsUp.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/app.whatsup.WhatsUp.svg
%{_datadir}/icons/hicolor/symbolic/apps/app.whatsup.WhatsUp-symbolic.svg
%{_datadir}/icons/hicolor/symbolic/apps/app.whatsup.WhatsUp-unread-symbolic.svg
%{_datadir}/icons/hicolor/scalable/apps/app.whatsup.WhatsUp-tray.svg
%{_datadir}/icons/hicolor/scalable/apps/app.whatsup.WhatsUp-tray-unread.svg

%changelog
* Sat Oct 17 2026 Maarten van der Rijst <claude@feedmebits.nl> - 1.0.0-1
- First release: WhatsApp Web in a native GNOME window, with multiple
  accounts, a tray icon, notifications, downloads, settings, editable
  shortcuts, and colours from the system theme

* Sat Oct 17 2026 Maarten van der Rijst <claude@feedmebits.nl> - 0.1.0-1
- First package
