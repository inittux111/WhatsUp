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

- **Name:** "WhatsUp" for the app as the user sees it, the repo folder (`~/Git/claude/whatsup-desktop`) and the GitHub repo (`inittux111/whatsup-desktop`). App ID `app.whatsup.WhatsUp`. Only the crate, binary and data folder are lowercase `whatsup-desktop`, as Rust and Linux conventions expect.
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

- `app.rs` starts the `adw::Application` (single instance: a second launch presents the existing window). `window.rs` builds the window: a header bar with the main menu above the web view.
- `web/` owns everything about the web content and is the trust boundary: `policy.rs` decides which navigations load inside (only `https://web.whatsapp.com`) and which links may be handed to the desktop (`http`, `https`, `mailto`); `permissions.rs` allows only notifications and camera/microphone, and only for WhatsApp; `style.rs` injects the user's stylesheets. Anything new that reacts to page content (script messages, downloads, notifications) belongs here and must go through the same checks.
- `background.rs` makes the app keep running when the window closes (it only hides) and feeds the unread count, read from WhatsApp's page title by `unread.rs`, to the tray and to `badge.rs` (the Unity launcher count that docks such as Dash to Dock show). `Ctrl+Q` and the tray's Quit really quit.
- `notifications.rs` turns WhatsApp Web's notifications into GNOME notifications (`GNotification`, so Do Not Disturb and per-app settings apply); clicking one presents the window and calls WebKit's `clicked()` so WhatsApp opens the chat. GNOME only shows them for an app whose `.desktop` file is installed, so test notifications with the RPM installed. `web/permissions.rs` pre-grants notification permission to WhatsApp's origin, so WhatsApp never asks.
- `tray/` is the tray icon, implemented directly on GIO's D-Bus (no crate): `item.rs` is `org.kde.StatusNotifierItem` (GNOME shows it through the AppIndicator extension), `menu.rs` is the `com.canonical.dbusmenu` menu, and `mod.rs` registers both with `org.kde.StatusNotifierWatcher`, again whenever the watcher reappears. Callbacks into the app run from an idle handler, never inside a D-Bus call. Debug builds point the icon theme path at `data/icons`, so the icons work without installing.
- `paths.rs` is the only place that decides file locations, and creates them 0700. Each account has its own WebKit `NetworkSession` in `~/.local/share/whatsup-desktop/accounts/<account>` (cookies, login) and `~/.cache/whatsup-desktop/accounts/<account>`; so far there is one account, `default`.
- Web stylesheets are every `.css` file in `~/.config/whatsup-desktop/css`, read asynchronously at startup and injected as user-level style sheets for `web.whatsapp.com` only. Abyss is applied by symlinking its stylesheet there: `ln -s ~/Git/claude/abyss-theme/src/zapzap/Abyss.css ~/.config/whatsup-desktop/css/abyss.css`. Its light and dark sets follow the page's `prefers-color-scheme`, which WebKitGTK takes from libadwaita.
- `data/` holds the desktop file, metainfo and icons; `whatsup-desktop.spec` and the `Makefile` package them.

## Theming (Abyss)

The app must follow the user's Abyss theme, which lives in `../abyss-theme`:

- Abyss themes libadwaita apps by linking `~/.config/gtk-4.0/gtk.css` to the current scheme's stylesheet. A libadwaita app picks that up automatically, so the app must not override it: use libadwaita's named colors (`@accent_bg_color`, `@window_bg_color`, ...) in app CSS, never hardcoded colors, and don't force a `GTK_THEME` or color scheme.
- For the web content, Abyss already ships a WhatsApp Web stylesheet for ZapZap at `../abyss-theme/src/zapzap/Abyss.css` (installed by `src/apps/zapzap.sh`). Reuse it rather than writing a new one.

## Development container

All building and development happens in a container: never install build tools or `-devel` packages on the host. Use a dedicated toolbox, `whatsup-desktop`, on Fedora 44 like the host, so RPMs built in it match the host's libraries. Inside it, anything development needs is allowed without asking, including installing and removing packages. Leave the user's other toolboxes (`ba`, `fc44`) alone.

```sh
toolbox create whatsup-desktop --image registry.fedoraproject.org/fedora-toolbox:44   # if it doesn't exist
toolbox run -c whatsup-desktop sudo dnf install -y cargo clippy rustfmt libadwaita-devel \
    webkitgtk6.0-devel blueprint-compiler rpm-build google-noto-color-emoji-fonts \
    desktop-file-utils libappstream-glib make
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

The app ships as an RPM: `whatsup-desktop.spec`, the `.desktop` file, the icons and the app ID, all matching. `toolbox run -c whatsup-desktop make rpm` packs the sources and the vendored crates (`cargo vendor`, so `rpmbuild` builds offline) into `target/rpm/SOURCES` and builds the RPM into `target/rpm/RPMS/x86_64/`. The spec must `Requires: webkitgtk6.0`, `libadwaita` and `google-noto-color-emoji-fonts`. Keep `BuildRequires`/`Requires` in sync with what the toolbox needs. The user installs the RPM on the host (`sudo dnf install ./<file>.rpm`).

## Verification

The app is done when the user can do everything in it that they can do in ZapZap. Automated tests cover the logic, but feature parity is checked by hand. When you finish a phase or feature, tell the user what to try and how it should behave. Commit after the user confirms a phase; push only when asked.
