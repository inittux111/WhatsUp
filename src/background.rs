//! Running in the background: closing the window hides it, the tray icon
//! brings it back, and the unread count shows on the tray and the dock.

use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;
use webkit::prelude::*;

use crate::{badge, paths, tray, unread};

pub fn attach(app: &adw::Application, window: &adw::ApplicationWindow, view: &webkit::WebView) {
    window.connect_close_request(|window| {
        window.set_visible(false);
        glib::Propagation::Stop
    });

    let Some(connection) = app.dbus_connection() else {
        return;
    };
    let tray = new_tray(app, window, &connection).map(Rc::new);

    if let Some(tray) = &tray {
        tray.set_window_visible(window.is_visible());
        let tray = tray.clone();
        window.connect_visible_notify(move |window| tray.set_window_visible(window.is_visible()));
    }

    view.connect_title_notify(move |view| {
        let count = view
            .title()
            .map_or(0, |title| unread::count_from_title(&title));
        if let Some(tray) = &tray {
            tray.set_unread(count);
        }
        badge::set_count(&connection, count);
    });
}

fn new_tray(
    app: &adw::Application,
    window: &adw::ApplicationWindow,
    connection: &gtk::gio::DBusConnection,
) -> Option<tray::Tray> {
    let icon_theme_path = paths::dev_icon_dir()
        .map(|dir| dir.to_string_lossy().into_owned())
        .unwrap_or_default();
    let toggle = glib::clone!(
        #[weak]
        window,
        move || toggle(&window)
    );
    let quit = glib::clone!(
        #[weak]
        app,
        move || app.quit()
    );
    match tray::Tray::new(connection, icon_theme_path, toggle, quit) {
        Ok(tray) => Some(tray),
        Err(err) => {
            glib::g_warning!("whatsup-desktop", "no tray icon: {}", err.message());
            None
        }
    }
}

/// Hides the window when it's in front, and brings it to the front otherwise.
fn toggle(window: &adw::ApplicationWindow) {
    if window.is_visible() && window.is_active() {
        window.set_visible(false);
    } else {
        window.present();
    }
}
