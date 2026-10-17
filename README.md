# WhatsUp

WhatsApp Web in a native GNOME window. WhatsUp runs WhatsApp's official web
client in WebKitGTK, inside a GTK4 + libadwaita app written in Rust for speed
and a light footprint.

## Features

- **Multiple accounts**, each with its own login; switch between them from the
  title bar, which shows each account's unread count
- **Tray icon** with the unread count, in green or matching your top bar, and a
  menu with Show/Hide, Settings and Quit
- **Runs in the background** when you close the window; optionally starts at
  login, hidden in the tray
- **GNOME notifications** that follow Do Not Disturb; clicking one opens the
  chat; message previews can be turned off
- **Downloads** to a folder of your choice, never overwriting a file, or asks
  where to save each one
- **Zoom**, **spell checking** in the languages you pick, and **keyboard
  shortcuts** you can change
- **Follows your GNOME theme**: WhatsApp itself takes its colours from your
  theme and accent colour, light or dark, and changes along with them; your
  own stylesheets and (if you turn them on) scripts go on top
- Remembers its window size, and shows the unread count on the app icon in
  docks that support it

## Installing

Build the RPM (see below) and install it:

```sh
sudo dnf install ./target/rpm/RPMS/x86_64/whatsup-desktop-*.rpm
```

## Customizing

| What | Where |
|---|---|
| Stylesheets for WhatsApp Web | `~/.config/whatsup-desktop/css/*.css` |
| Scripts for WhatsApp Web (off by default) | `~/.config/whatsup-desktop/js/*.js` |
| Settings | `~/.config/whatsup-desktop/settings.ini` (or Settings, `Ctrl+,`) |

WhatsApp's colours come from your GNOME theme (Settings → Chats → Use System
Colours). Stylesheets go on top of that, in name order. Settings → Chats →
Styles and Scripts opens these folders and reloads them without a restart.

## Building from source

Development happens in a Fedora toolbox, so nothing is installed on the host:

```sh
toolbox create whatsup-desktop --image registry.fedoraproject.org/fedora-toolbox:44
toolbox run -c whatsup-desktop sudo dnf install -y cargo clippy rustfmt libadwaita-devel \
    webkitgtk6.0-devel rpm-build google-noto-color-emoji-fonts \
    desktop-file-utils libappstream-glib make hunspell-en-US

toolbox run -c whatsup-desktop cargo run       # run the app
toolbox run -c whatsup-desktop cargo test      # run the tests
toolbox run -c whatsup-desktop make rpm        # build the RPM into target/rpm/RPMS/
```

Notifications and starting at login need the installed RPM, as GNOME only
shows notifications for apps with an installed `.desktop` file.

## Privacy

- Only `web.whatsapp.com` loads inside the app; other links open in your
  browser.
- WhatsApp Web may only use notifications, the camera and the microphone.
- Each account's data stays in private folders (`~/.local/share/whatsup-desktop` and
  `~/.cache/whatsup-desktop`, readable only by you).

## Licence

GPL-3.0-or-later
