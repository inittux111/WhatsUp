//! WhatsUp: WhatsApp Web in a native GNOME window.

mod app;
mod background;
mod badge;
mod notifications;
mod paths;
mod tray;
mod unread;
mod web;
mod window;

use gtk::glib;

fn main() -> glib::ExitCode {
    app::run()
}
