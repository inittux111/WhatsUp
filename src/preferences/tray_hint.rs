//! A hint under Show Tray Icon while the desktop has no tray to show it
//! (no StatusNotifierWatcher on the session bus). Debian turns the MATE
//! panel's support for such icons off (org.mate.panel enable-sni-support),
//! so on MATE the hint explains how to turn it on, in a terminal or in
//! dconf Editor, and offers to do it.

use std::cell::RefCell;
use std::ffi::OsStr;

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::{tray, web};

const MATE_SCHEMA: &str = "org.mate.panel";
const MATE_KEY: &str = "enable-sni-support";
const MATE_COMMANDS: &str =
    "gsettings set org.mate.panel enable-sni-support true\nmate-panel --replace &";

/// The hint row; it shows itself only while no tray is available.
pub fn row() -> gtk::Widget {
    let row = match mate_settings() {
        Some(settings) => mate_row(settings).upcast(),
        None => other_row().upcast(),
    };
    show_while_no_tray(&row);
    row
}

fn mate_row(settings: gio::Settings) -> adw::ExpanderRow {
    let row = adw::ExpanderRow::builder()
        .title("Tray Icon Not Showing")
        .subtitle("The MATE panel's support for tray icons like this one is turned off")
        .build();
    row.add_prefix(&warning_icon());

    let automatic = adw::ActionRow::builder()
        .title("Turn On Now")
        .subtitle("Turns on the panel setting and restarts the panel")
        .build();
    let turn_on = gtk::Button::builder()
        .label("Turn On")
        .valign(gtk::Align::Center)
        .css_classes(["suggested-action"])
        .build();
    turn_on.connect_clicked(move |button| match turn_on_mate(&settings) {
        Ok(()) => {
            button.set_sensitive(false);
            button.set_label("Restarting Panel…");
        }
        Err(err) => toast(button, &format!("Couldn't turn it on: {}", err.message())),
    });
    automatic.add_suffix(&turn_on);
    automatic.set_activatable_widget(Some(&turn_on));
    row.add_row(&automatic);

    let terminal = adw::ActionRow::builder()
        .title("In a Terminal")
        .subtitle(format!(
            "<tt>{}</tt>",
            glib::markup_escape_text(MATE_COMMANDS)
        ))
        .subtitle_selectable(true)
        .build();
    let copy = gtk::Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copy Commands")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    copy.connect_clicked(|button| {
        if let Some(display) = gdk::Display::default() {
            display.clipboard().set_text(MATE_COMMANDS);
            toast(button, "Commands copied");
        }
    });
    terminal.add_suffix(&copy);
    row.add_row(&terminal);

    row.add_row(
        &adw::ActionRow::builder()
            .title("With dconf Editor")
            .subtitle(
                "Install it with <tt>sudo apt install dconf-editor</tt>, open \
                 /org/mate/panel/, turn on enable-sni-support, then log out and back in",
            )
            .subtitle_selectable(true)
            .build(),
    );
    row
}

fn other_row() -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title("Tray Icon Not Showing")
        .subtitle(
            "Your desktop isn't showing tray icons like this one. On GNOME, turn on the \
             AppIndicator extension",
        )
        .build();
    row.add_prefix(&warning_icon());
    row
}

fn warning_icon() -> gtk::Image {
    gtk::Image::builder()
        .icon_name("dialog-warning-symbolic")
        .css_classes(["warning"])
        .build()
}

/// Shows `row` only while no StatusNotifierWatcher is on the bus, and stops
/// watching once the dialog is gone.
fn show_while_no_tray(row: &gtk::Widget) {
    row.set_visible(false);
    let (appeared, vanished) = (row.downgrade(), row.downgrade());
    let id = gio::bus_watch_name(
        gio::BusType::Session,
        tray::WATCHER,
        gio::BusNameWatcherFlags::NONE,
        move |_, _, _| {
            if let Some(row) = appeared.upgrade() {
                row.set_visible(false);
            }
        },
        move |_, _| {
            if let Some(row) = vanished.upgrade() {
                row.set_visible(true);
            }
        },
    );
    let id = RefCell::new(Some(id));
    row.connect_destroy(move |_| {
        if let Some(id) = id.take() {
            gio::bus_unwatch_name(id);
        }
    });
}

/// The MATE panel's settings, on MATE with a panel that has the setting.
fn mate_settings() -> Option<gio::Settings> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    if !is_mate(&desktop) || glib::find_program_in_path("mate-panel").is_none() {
        return None;
    }
    // Looked up first: gio::Settings::new aborts on a missing schema.
    let schema = gio::SettingsSchemaSource::default()?.lookup(MATE_SCHEMA, true)?;
    schema
        .has_key(MATE_KEY)
        .then(|| gio::Settings::new_full(&schema, None::<&gio::SettingsBackend>, None))
}

fn is_mate(current_desktop: &str) -> bool {
    current_desktop
        .split(':')
        .any(|name| name.eq_ignore_ascii_case("MATE"))
}

/// Turns the panel's support on and restarts the panel, which reads it only
/// at startup. The new panel gets the environment WhatsUp started with and
/// its own session, so it doesn't share WhatsUp's changes or its terminal.
fn turn_on_mate(settings: &gio::Settings) -> Result<(), glib::Error> {
    settings
        .set_boolean(MATE_KEY, true)
        .map_err(|err| glib::Error::new(gio::IOErrorEnum::Failed, &err.to_string()))?;
    gio::Settings::sync();
    let launcher = gio::SubprocessLauncher::new(
        gio::SubprocessFlags::STDOUT_SILENCE | gio::SubprocessFlags::STDERR_SILENCE,
    );
    launcher.set_environ(web::original_environment());
    let mut argv: Vec<&OsStr> = vec![OsStr::new("mate-panel"), OsStr::new("--replace")];
    if glib::find_program_in_path("setsid").is_some() {
        argv.insert(0, OsStr::new("setsid"));
    }
    let panel = launcher.spawn(&argv)?;
    // Reaps it, should it ever exit while WhatsUp runs.
    panel.wait_async(gio::Cancellable::NONE, |_| {});
    Ok(())
}

fn toast(widget: &impl IsA<gtk::Widget>, message: &str) {
    if let Some(dialog) = widget
        .ancestor(adw::PreferencesDialog::static_type())
        .and_downcast::<adw::PreferencesDialog>()
    {
        dialog.add_toast(adw::Toast::new(message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_mate() {
        assert!(is_mate("MATE"));
        assert!(is_mate("ubuntu:MATE"));
        assert!(!is_mate("GNOME"));
        assert!(!is_mate(""));
        assert!(!is_mate("MATE-ish"));
    }
}
