//! The Downloads group of the Chats page: where files are saved.

use std::path::Path;

use adw::prelude::*;
use gtk::{gio, glib};

use super::switch;
use crate::paths;
use crate::settings::{Settings, Values};

pub fn group(
    window: &adw::ApplicationWindow,
    settings: &Settings,
    values: &Values,
) -> adw::PreferencesGroup {
    let group = super::group("Downloads");
    let row = adw::ActionRow::builder().title("Download Folder").build();
    show_folder(&row, values.download_folder.as_deref());

    let choose = gtk::Button::builder()
        .icon_name("folder-open-symbolic")
        .tooltip_text("Choose Folder")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    choose.connect_clicked(glib::clone!(
        #[weak]
        window,
        #[weak]
        row,
        #[strong]
        settings,
        move |_| {
            glib::spawn_future_local(choose_folder(window, row, settings.clone()));
        }
    ));
    let reset = gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text("Use the Downloads Folder")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    reset.connect_clicked(glib::clone!(
        #[weak]
        row,
        #[strong]
        settings,
        move |_| {
            settings.update(|v| v.download_folder = None);
            show_folder(&row, None);
        }
    ));
    row.add_suffix(&reset);
    row.add_suffix(&choose);
    row.set_activatable_widget(Some(&choose));
    group.add(&row);
    group.add(&switch(
        "Ask Where to Save",
        "Choose a folder for each file instead",
        values.ask_download_location,
        settings,
        |v, on| v.ask_download_location = on,
    ));
    group
}

/// Asks for a folder through the desktop's folder picker, without blocking.
async fn choose_folder(window: adw::ApplicationWindow, row: adw::ActionRow, settings: Settings) {
    let dialog = gtk::FileDialog::builder()
        .title("Download Folder")
        .modal(true)
        .build();
    let current = settings
        .get()
        .download_folder
        .unwrap_or_else(paths::downloads_dir);
    dialog.set_initial_folder(Some(&gio::File::for_path(current)));
    // Cancelling the picker is an error too; it changes nothing.
    let Ok(folder) = dialog.select_folder_future(Some(&window)).await else {
        return;
    };
    if let Some(path) = folder.path().filter(|path| path.is_absolute()) {
        show_folder(&row, Some(&path));
        settings.update(|v| v.download_folder = Some(path));
    }
}

fn show_folder(row: &adw::ActionRow, folder: Option<&Path>) {
    let folder = folder.map_or_else(paths::downloads_dir, Path::to_path_buf);
    let home = glib::home_dir();
    let shown = match folder.strip_prefix(&home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => folder.display().to_string(),
    };
    row.set_subtitle(&glib::markup_escape_text(&shown));
}
