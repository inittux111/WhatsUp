//! The unread count on the app's icon in docks that show one (Dash to Dock,
//! Ubuntu Dock, ...), through the Unity launcher API.

use gtk::gio;
use gtk::glib::{self, Variant, prelude::*};
use std::collections::HashMap;

const OBJECT_PATH: &str = "/app/whatsup_desktop/WhatsUp/LauncherEntry";
const APP_URI: &str = "application://app.whatsup_desktop.WhatsUp.desktop";
/// The name docks own while they listen for launcher entries.
const DOCK: &str = "com.canonical.Unity";

/// Calls `f` whenever a dock that shows counts appears, so the count can be
/// sent again: a dock forgets it when it restarts, and GNOME restarts its
/// extensions (Dash to Dock among them) on every unlock.
pub fn connect_dock_appeared(connection: &gio::DBusConnection, f: impl Fn() + 'static) {
    // The watch lasts as long as the app; the id is deliberately never unwatched.
    let _ = gio::bus_watch_name_on_connection(
        connection,
        DOCK,
        gio::BusNameWatcherFlags::NONE,
        move |_, _, _| f(),
        |_, _| {},
    );
}

pub fn set_count(connection: &gio::DBusConnection, count: u32) {
    let props = HashMap::from([
        ("count", i64::from(count).to_variant()),
        ("count-visible", (count > 0).to_variant()),
    ]);
    let params: Variant = (APP_URI, props).to_variant();
    if let Err(err) = connection.emit_signal(
        None,
        OBJECT_PATH,
        "com.canonical.Unity.LauncherEntry",
        "Update",
        Some(&params),
    ) {
        glib::g_debug!(
            "whatsup-desktop",
            "cannot update the dock badge: {}",
            err.message()
        );
    }
}
