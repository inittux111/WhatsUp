//! Starting at login, through an XDG autostart entry in
//! `~/.config/autostart` (created if missing; `~/.config` always exists),
//! written in the background.

use std::cell::Cell;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::app::{APP_ID, APP_NAME};

/// The flag the autostart entry passes to start hidden in the tray.
pub const BACKGROUND_FLAG: &str = "background";
/// The app ID of early builds, which named their entry after it.
const OLD_APP_ID: &str = "app.whatsup.WhatsUp";

thread_local! {
    /// The entry the latest change asked for, (enabled, hidden), until it's written.
    static WANTED: Cell<Option<(bool, bool)>> = const { Cell::new(None) };
    /// Whether a change is being written; the next waits for it.
    static BUSY: Cell<bool> = const { Cell::new(false) };
}

/// Writes or removes the entry. Changes are written one at a time, in order,
/// so quickly turning it on and off can't leave the entry in place.
pub fn apply(enabled: bool, hidden: bool) {
    WANTED.set(Some((enabled, hidden)));
    if BUSY.replace(true) {
        return;
    }
    let file = gio::File::for_path(
        glib::user_config_dir()
            .join("autostart")
            .join(format!("{APP_ID}.desktop")),
    );
    glib::spawn_future_local(async move {
        while let Some((enabled, hidden)) = WANTED.take() {
            let result = if enabled {
                write(&file, entry(hidden)).await
            } else {
                remove(&file).await
            };
            if let Err(err) = result {
                glib::g_warning!(
                    "whatsup-desktop",
                    "cannot change starting at login: {}",
                    err.message()
                );
            }
        }
        BUSY.set(false);
    });
}

fn entry(hidden: bool) -> String {
    let flag = if hidden {
        format!(" --{BACKGROUND_FLAG}")
    } else {
        String::new()
    };
    format!(
        "[Desktop Entry]\nType=Application\nName={APP_NAME}\nExec=whatsup-desktop{flag}\n\
         Icon={APP_ID}\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
    )
}

async fn write(file: &gio::File, data: String) -> Result<(), glib::Error> {
    if let Some(dir) = file.parent()
        && let Err(err) = dir.make_directory_future(glib::Priority::DEFAULT).await
        && !err.matches(gio::IOErrorEnum::Exists)
    {
        return Err(err);
    }
    file.replace_contents_future(data.into_bytes(), None, false, gio::FileCreateFlags::NONE)
        .await
        .map(|_| ())
        .map_err(|(_, err)| err)
}

/// Deletes the entry an early build wrote under its old app ID, in the
/// background, so the app doesn't start twice at login.
pub fn remove_old() {
    let file = gio::File::for_path(
        glib::user_config_dir()
            .join("autostart")
            .join(format!("{OLD_APP_ID}.desktop")),
    );
    glib::spawn_future_local(async move {
        if let Err(err) = remove(&file).await {
            glib::g_warning!(
                "whatsup-desktop",
                "cannot remove the old login entry: {}",
                err.message()
            );
        }
    });
}

async fn remove(file: &gio::File) -> Result<(), glib::Error> {
    match file.delete_future(glib::Priority::DEFAULT).await {
        Err(err) if !err.matches(gio::IOErrorEnum::NotFound) => Err(err),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_starts_hidden_only_when_asked() {
        assert!(entry(true).contains("Exec=whatsup-desktop --background\n"));
        assert!(entry(false).contains("Exec=whatsup-desktop\n"));
        assert!(entry(false).starts_with("[Desktop Entry]\n"));
    }
}
