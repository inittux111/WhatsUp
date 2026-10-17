//! The application: one instance per session, one main window.

use adw::prelude::*;
use gtk::glib;

use crate::{background, downloads, notifications, paths, window};

pub const APP_ID: &str = "app.whatsup.WhatsUp";
pub const APP_NAME: &str = "WhatsUp";

pub fn run() -> glib::ExitCode {
    glib::set_application_name(APP_NAME);
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| add_dev_icons());
    app.connect_activate(activate);
    app.set_accels_for_action("win.reload", &["<Control>r", "F5"]);
    app.set_accels_for_action("app.quit", &["<Control>q"]);

    let quit = gtk::gio::ActionEntry::builder("quit")
        .activate(|app: &adw::Application, _, _| app.quit())
        .build();
    app.add_action_entries([quit]);

    app.run()
}

/// Shows the existing window, or builds it on first launch.
fn activate(app: &adw::Application) {
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }
    match window::build(app) {
        Ok(main) => {
            background::attach(app, &main.window, &main.view);
            notifications::attach(app, &main.window, &main.view);
            downloads::attach(&main.view, &main.toasts);
            main.window.present();
        }
        Err(err) => {
            eprintln!("{APP_NAME}: cannot prepare the account folder: {err}");
            app.quit();
        }
    }
}

/// In development builds, finds the app's icons in the source tree.
fn add_dev_icons() {
    if let (Some(dir), Some(display)) = (paths::dev_icon_dir(), gtk::gdk::Display::default()) {
        gtk::IconTheme::for_display(&display).add_search_path(dir);
    }
}
