//! The Styles and Scripts group of the Chats page: the user's own
//! stylesheets and scripts for WhatsApp Web, in WhatsUp's config folder.

use std::path::{Path, PathBuf};

use adw::prelude::*;
use gtk::{gio, glib};

use super::{group, switch};
use crate::paths;
use crate::settings::{Settings, Values};

pub fn group_for(settings: &Settings, values: &Values) -> adw::PreferencesGroup {
    let group = group("Styles and Scripts");
    group.set_description(Some(
        "Files in these folders are added to WhatsApp Web, in name order.",
    ));
    group.add(&folder_row(
        "Stylesheets",
        "CSS files, such as a theme",
        paths::css_dir(),
    ));
    group.add(&folder_row("Scripts", "JavaScript files", paths::js_dir()));
    group.add(&switch(
        "Run Custom Scripts",
        "Scripts can read and send your messages: only use ones you trust",
        values.custom_scripts,
        settings,
        |v, on| v.custom_scripts = on,
    ));
    let reload = adw::ButtonRow::builder()
        .title("Reload Styles and Scripts")
        .start_icon_name("view-refresh-symbolic")
        .build();
    reload.connect_activated(|row| {
        let _ = row.activate_action("win.reload-customizations", None);
    });
    group.add(&reload);
    group
}

fn folder_row(title: &str, subtitle: &str, folder: PathBuf) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(format!(
            "{subtitle} in {}",
            glib::markup_escape_text(&shown(&folder))
        ))
        .build();
    let open = gtk::Button::builder()
        .icon_name("folder-open-symbolic")
        .tooltip_text("Open Folder")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    open.connect_clicked(move |button| open_folder(button, &folder));
    row.add_suffix(&open);
    row.set_activatable_widget(Some(&open));
    row
}

/// Opens `folder` in the file manager, creating it (private) first.
fn open_folder(button: &gtk::Button, folder: &Path) {
    if let Err(err) = paths::ensure_private(folder) {
        glib::g_warning!("whatsup-desktop", "cannot create a folder: {err}");
        return;
    }
    let window = button.root().and_downcast::<gtk::Window>();
    gtk::FileLauncher::new(Some(&gio::File::for_path(folder))).launch(
        window.as_ref(),
        gio::Cancellable::NONE,
        |result| {
            if let Err(err) = result {
                glib::g_warning!("whatsup-desktop", "cannot open a folder: {}", err.message());
            }
        },
    );
}

/// `folder` with the home folder as "~".
fn shown(folder: &Path) -> String {
    match folder.strip_prefix(glib::home_dir()) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => folder.display().to_string(),
    }
}
