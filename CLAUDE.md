# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

WhatsUp is a WhatsApp client for Linux/GNOME, for personal use only (not distributed), packaged as an RPM for the user's own Fedora system.

- **Approach:** a Rust app that embeds WhatsApp Web in **WebKitGTK** (`webkitgtk6.0`, through the `webkit6` crate), so the account uses WhatsApp's official web client and isn't at risk of a ban. The app around it is native, built with GTK4 + libadwaita (gtk-rs): window, tray icon, notifications, settings, accounts. WebKitGTK was chosen over Chromium (CEF): it's GNOME's GTK4-native engine, already on the system and far easier to package.
- **No unofficial protocol libraries:** an earlier fully native version built on one was abandoned over the account-ban risk. Don't go back to that approach.
- **Reference for "done":** ZapZap (https://github.com/rafatosta/zapzap): all of its features, built independently in Rust (not a port of its code). It should look and feel like a modern app made by a professional designer: follow the GNOME Human Interface Guidelines and libadwaita's patterns rather than custom styling.
- **Why it exists:** ZapZap stops responding after switching between a game and another workspace, most of all when attaching and sending a file in a chat while a game runs. WhatsUp must stay responsive then; the attachment flow is the key case to test while gaming.
- **Autonomy:** make reasonable choices yourself and list your assumptions at the end. Ask the user only about decisions that are hard to reverse.
- **Workflow:** use the available coding skills while coding, including the Superpowers plugin (enabled in `.claude/settings.json`). Work as a professional software engineer and a professional UI/UX designer would: clean, well-structured, tested code, and an interface that is polished, consistent and easy to use.

## Rules from the user

- **Name:** "WhatsUp" for the app as the user sees it. App ID `app.whatsup_desktop.WhatsUp`. Everything else is lowercase `whatsup-desktop`, as Rust and Linux conventions expect: the crate, binary, packages, data folders, toolbox, repo folder (`~/Git/claude/whatsup-desktop`) and GitHub repo (`inittux111/whatsup-desktop`). Not plain `whatsup`: Fedora already has an unrelated package by that name (a cluster tool that also installs `/usr/bin/whatsup`).
- **Files stay under 500 lines.** Split by responsibility before a file reaches that.
- **Few dependencies.** Add a crate only when it's needed: when writing it ourselves would be less safe (security) or noticeably slower (performance). Prefer GTK/GLib/GIO and the standard library. Before adding a crate, check its maintenance, licence and known security advisories ([RustSec](https://rustsec.org)); ask the user before adding one with a real cost. When `cargo audit` reports a security problem in a dependency, update it right away, or replace or remove it if no fixed version exists.
- **Secure and performant:** see the sections below.
- **Modular:** small modules with one responsibility each, so new features can be added later without reworking existing code.
- **Easy to install and package:** few runtime libraries beyond what Fedora's GNOME ships, built into an RPM.

## Security

- Keep account data private (0700 folders). Never log message contents, phone numbers or QR codes.
- Treat everything from the web content as untrusted: validate every message the page sends to the app before acting on it.
- Only allow `web.whatsapp.com` inside the web view; open every other link in the user's default browser.
- Grant camera, microphone and notification permission only to WhatsApp's own origin; refuse all other permission requests.
- WebKitGTK 6 always runs web content in its sandbox; don't weaken it (e.g. with `add_path_to_sandbox`) without need. The web inspector (developer tools) is on in debug builds only.
- Sanitize download file names before saving, so a file can't be written outside the Downloads folder.

## Performance

- Nothing slow on the GTK main thread: do disk and network work asynchronously (GIO async APIs or a background thread), never in the code that draws the window.
- Open files through the GNOME file picker via the desktop portal, asynchronously, so the app never blocks while it's open. This is exactly where ZapZap freezes.
- Keep the app light when it's hidden or minimised: don't redraw or do work for anything that isn't visible.
- Test the attach-and-send flow with a game running, as part of every phase.

## Architecture

- `app.rs` starts the `adw::Application` (single instance: a second launch presents the existing window; `--background` keeps the first window hidden, as starting at login does). `window.rs` builds the window (a header bar with the main menu above a toast overlay holding an `adw::ViewStack` with one page per account; with several accounts the header shows a view switcher with unread badges) and returns `window::Main`, which the feature modules (`background`, `notifications`, `downloads`) take in their `attach`.
- `accounts.rs` owns the open accounts: one `Session` (web view, stack page, unread count) per account in the settings, kept in sync as accounts are added, renamed or removed (removing one terminates its web process and deletes its data via `paths::remove_account_data`). Features never hold a single web view: they register with `Accounts::on_session`, which runs for every page, including ones added later, and read the total unread count with `connect_unread`.
- `settings/` holds the user's settings: `values.rs` is the `Values` struct and its key-file format (`~/.config/whatsup-desktop/settings.ini`, 0600, invalid values fall back to defaults), `mod.rs` loads it once at startup, saves in the background on every change and calls `connect_changed` listeners, and `autostart.rs` writes or removes `~/.config/autostart/app.whatsup_desktop.WhatsUp.desktop`. Features read `settings.get()` when they act, or listen for changes (zoom and spell checking in `web::follow_settings`). `preferences/` is the Settings dialog (only the app's own settings; WhatsApp's stay in its page; rows write their setting immediately), one file per page: `general.rs`, `chats.rs` (with `downloads.rs` and the installed dictionaries from `languages.rs`) and `keys.rs`, the shortcut editor. `shortcuts.rs` is the one list of keyboard shortcuts with their defaults; the user's changes live in `Values::shortcuts` (`""` turns one off) and must pass `settings::accel::is_safe` (Ctrl, Alt or Super, or an F-key). It registers them as accelerators (shown in menus), handles them in the window's capture phase so WhatsApp Web can't swallow them while typing, and is suspended while the editor records keys. The window size is saved on close and quit; quitting saves synchronously (`Settings::save_now`), since a background save wouldn't finish.
- `web/` owns everything about the web content and is the trust boundary: `policy.rs` decides which navigations load inside (only `https://web.whatsapp.com`) and which links may be handed to the desktop (`http`, `https`, `mailto`); `permissions.rs` allows only notifications and camera/microphone, and only for WhatsApp; `style.rs` injects the user's stylesheets. Anything new that reacts to page content (script messages, downloads, notifications) belongs here and must go through the same checks.
- `background.rs` makes the app keep running when the window closes (it only hides) and feeds the unread count, read from WhatsApp's page title by `unread.rs`, to the tray and to `badge.rs` (the Unity launcher count that docks such as Dash to Dock show). `Ctrl+Q` and the tray's Quit really quit.
- `notifications.rs` turns WhatsApp Web's notifications into GNOME notifications (`GNotification`, so Do Not Disturb and per-app settings apply); clicking one presents the window and calls WebKit's `clicked()` so WhatsApp opens the chat. GNOME only shows them for an app whose `.desktop` file is installed, so test notifications with the RPM installed. `web/permissions.rs` pre-grants notification permission to WhatsApp's origin, so WhatsApp never asks.
- `downloads/` saves files from chats to the Downloads folder: `filename.rs` cleans the page's suggested name (no folders, hidden names or control characters) and picks an unused one (`name (1).ext`); `mod.rs` shows a toast with Open when it's saved. `web/policy.rs` turns WhatsApp's `blob:https://web.whatsapp.com/...` links and anything WebKit can't display into downloads.
- `tray/` is the tray icon, implemented directly on GIO's D-Bus (no crate): `item.rs` is `org.kde.StatusNotifierItem` (GNOME shows it through the AppIndicator extension), `menu.rs` is the `com.canonical.dbusmenu` menu (Show/Hide, Settings, Quit), and `mod.rs` registers both with `org.kde.StatusNotifierWatcher`, again whenever the watcher reappears. Callbacks into the app run from an idle handler, never inside a D-Bus call. Debug builds point the icon theme path at `data/icons`, so the icons work without installing.
- `paths.rs` is the only place that decides file locations, and creates them 0700. Each account has its own WebKit `NetworkSession` in `~/.local/share/whatsup-desktop/accounts/<id>` (cookies, login) and `~/.cache/whatsup-desktop/accounts/<id>`. Account ids (`settings/accounts.rs`) are lowercase letters, digits and dashes only (`is_id`), checked on every read, so they can never point outside those folders; the first account is `default`.
- `web/style.rs` injects, for `web.whatsapp.com` only: the app's own fixes (`app.css`), the theme colors from `theme/` (`Extras::theme`), every `.css` file in `~/.config/whatsup-desktop/css`, and, only when the user turned on Run Custom Scripts, every `.js` file in `~/.config/whatsup-desktop/js`. Files are read asynchronously and swapped in at once; `win.reload-customizations` reloads them and the pages.
- `data/` holds the desktop file, metainfo and icons; `packaging/fedora/whatsup-desktop.spec` and the `Makefile` package them.

## Theming

The app must follow the active system theme, whatever it is. The user's own theme is Abyss (`../abyss-theme`), which themes libadwaita apps by linking `~/.config/gtk-4.0/gtk.css` to the current scheme's stylesheet:

- The native parts are plain libadwaita, so they pick up any theme automatically; the app must not override it: use libadwaita's named colors (`@accent_bg_color`, `@window_bg_color`, ...) in app CSS, never hardcoded colors, and don't force a `GTK_THEME` or color scheme.
- WhatsApp Web gets the theme's colors from `theme/`: invisible probe widgets each take one libadwaita color variable (`--window-bg-color`, `--accent-bg-color`, ...) as their text color, `theme/mod.rs` reads them back (again on light/dark, accent and contrast changes), and `theme/css.rs` maps them onto WhatsApp's `--WDS-*` variables. This works for Adwaita, Abyss and any other libadwaita theme, without a theme-specific stylesheet (setting: Use System Colours). User stylesheets, such as Abyss's own `../abyss-theme/src/zapzap/Abyss.css`, go on top.

## Development container

All building and development happens in a container: never install build tools or `-devel` packages on the host. Use a dedicated toolbox, `whatsup-desktop`, on Fedora 44 like the host, so RPMs built in it match the host's libraries. Inside it, anything development needs is allowed without asking, including installing and removing packages. Leave the user's other toolboxes (`ba`, `fc44`) alone.

```sh
toolbox create whatsup-desktop --image registry.fedoraproject.org/fedora-toolbox:44   # if it doesn't exist
toolbox run -c whatsup-desktop sudo dnf install -y cargo clippy rustfmt libadwaita-devel \
    webkitgtk6.0-devel blueprint-compiler rpm-build google-noto-color-emoji-fonts \
    desktop-file-utils libappstream-glib make hunspell-en-US
toolbox run -c whatsup-desktop cargo install cargo-audit                             # dependency security checks
```

The toolbox has its own fonts: without the emoji font, emoji show as white outlines there but not on the host. Toolbox shares the home directory and the Wayland session, so the repo is at the same path inside and `cargo run` opens the window on the host desktop.

## Commands

Run everything through the toolbox:

```sh
toolbox run -c whatsup-desktop cargo build                 # build
toolbox run -c whatsup-desktop cargo run                   # run the app
toolbox run -c whatsup-desktop cargo test                  # all tests
toolbox run -c whatsup-desktop cargo test <name>           # tests whose name matches <name>
toolbox run -c whatsup-desktop cargo clippy --all-targets  # lint
toolbox run -c whatsup-desktop cargo fmt                   # format
toolbox run -c whatsup-desktop cargo audit                 # check dependencies for security advisories
```

Officially supported: Fedora only; Debian and Ubuntu, Arch and openSUSE build too but are only documented, not tested per release. `make install` (`PREFIX`, `DESTDIR`, `BINARY`) is the one list of installed files; every package uses it, so a new file only needs adding there (and to the spec's `%files`).

- **Fedora (RPM):** `packaging/fedora/whatsup-desktop.spec`, built in the toolbox with `toolbox run -c whatsup-desktop make rpm`. It packs the committed code (`git archive`, so commit first) and the vendored crates (`cargo vendor`, so `rpmbuild` builds offline) into `target/rpm/SOURCES` and builds into `target/rpm/RPMS/x86_64/`. The spec must `Requires: webkitgtk6.0`, `libadwaita` and `google-noto-color-emoji-fonts` (openSUSE names differ, under `%if 0%{?suse_version}`; use `%{_datadir}/metainfo`, as openSUSE lacks `%{_metainfodir}`). Keep `BuildRequires`/`Requires` in sync with what the toolbox needs.
- **Debian and Ubuntu (deb):** built with Podman from any host: `make container-deb` builds the Ubuntu 26.04 image (`packaging/debian/Containerfile`, the oldest Debian-based release new enough: libadwaita 1.8+, GTK 4.20+, Rust edition 2024) and runs `make deb` (`packaging/debian/build.sh`) in it, as the current user, into `target/deb/`. The script works out `Depends` with `dpkg-shlibdeps`. It uses `target/deb-build` and `target/deb-cargo`, apart from the toolbox's `target/`. Debian 13 can't run WhatsUp (libadwaita 1.7).
- **Arch:** `packaging/arch/PKGBUILD` builds from the source tree (`makepkg -si` in that folder).
- **Versions:** a release sets the same version in `Cargo.toml`, `packaging/fedora/whatsup-desktop.spec` (plus a `%changelog` entry), `packaging/arch/PKGBUILD` and a `<release>` in the metainfo (the deb takes its revision from the spec's `Release`); `make rpm` refuses mismatched `Cargo.toml` and spec versions. The spec's `Release` stays `1`: a packaging-only change (spec, `Requires`, a rebuild) ships as a new patch version with its own tag, never as a `Release` bump. The COPR repo builds from its own spec in `inittux111/pkgs-upstream-latest`, whose daily job follows the latest GitHub release and resets `Release` to 1, so this keeps both at `X.Y.Z-1`.

The user installs the package on the host (`sudo dnf install ./<file>.rpm`).

## Verification

The app is done when the user can do everything in it that they can do in ZapZap. Automated tests cover the logic, but feature parity is checked by hand. When you finish a phase or feature, tell the user what to try and how it should behave. Commit after the user confirms a phase; push only when asked.
