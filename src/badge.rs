//! The unread count on the app's icon in docks that show one (Dash to Dock,
//! Ubuntu Dock, ...), through the Unity launcher API.

use gtk::gio;
use gtk::glib::{self, Variant, prelude::*};
use std::collections::HashMap;

const OBJECT_PATH: &str = "/app/whatsup_desktop/WhatsUp/LauncherEntry";
const APP_URI: &str = "application://app.whatsup_desktop.WhatsUp.desktop";

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
