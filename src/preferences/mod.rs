//! The Settings dialog: settings of the app itself, not of WhatsApp (those
//! stay in WhatsApp's own menu). Every row writes its setting straight away,
//! as GNOME apps do; there's no Apply button.

mod accounts;
mod chats;
mod custom;
mod downloads;
mod general;
mod keys;
mod languages;

use adw::prelude::*;

use crate::settings::{Settings, Values};

pub fn show(window: &adw::ApplicationWindow, settings: &Settings) {
    let values = settings.get();
    let dialog = adw::PreferencesDialog::builder()
        .title("Settings")
        .search_enabled(true)
        .build();
    dialog.add(&general::page(settings, &values));
    dialog.add(&chats::page(window, settings, &values));
    dialog.add(&accounts::page(settings));
    if let Some(app) = window.application().and_downcast::<adw::Application>() {
        dialog.add(&keys::page(&app, settings));
    }
    dialog.present(Some(window));
}

fn new_page(title: &str, icon: &str) -> adw::PreferencesPage {
    adw::PreferencesPage::builder()
        .title(title)
        .icon_name(icon)
        .build()
}

fn group(title: &str) -> adw::PreferencesGroup {
    adw::PreferencesGroup::builder().title(title).build()
}

/// A switch row that writes its setting with `set` whenever it's toggled.
fn switch(
    title: &str,
    subtitle: &str,
    active: bool,
    settings: &Settings,
    set: fn(&mut Values, bool),
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(title)
        .subtitle(subtitle)
        .active(active)
        .build();
    let settings = settings.clone();
    row.connect_active_notify(move |row| settings.update(|v| set(v, row.is_active())));
    row
}

/// Makes `row` usable only while `switch` is on.
fn depends_on(row: &impl IsA<gtk::Widget>, switch: &adw::SwitchRow) {
    switch
        .bind_property("active", row, "sensitive")
        .sync_create()
        .build();
}
