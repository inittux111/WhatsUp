//! WhatsUp: WhatsApp Web in a native GNOME window.

mod app;
mod paths;
mod web;
mod window;

use gtk::glib;

fn main() -> glib::ExitCode {
    app::run()
}
