//! Files saved from chats go to the Downloads folder under a cleaned-up,
//! unused name; a toast says when each one is saved and offers to open it.

mod filename;

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gio, glib};
use webkit::prelude::*;

use crate::paths;

pub fn attach(view: &webkit::WebView, toasts: &adw::ToastOverlay) {
    let Some(session) = view.network_session() else {
        return;
    };
    session.connect_download_started(glib::clone!(
        #[weak]
        toasts,
        move |_, download| track(download, &toasts)
    ));
}

fn track(download: &webkit::Download, toasts: &adw::ToastOverlay) {
    download.set_allow_overwrite(false);
    download.connect_decide_destination(|download, suggested| {
        choose_destination(download, suggested);
        true
    });

    let failed = Rc::new(Cell::new(false));
    download.connect_failed(glib::clone!(
        #[weak]
        toasts,
        #[strong]
        failed,
        move |download, err| {
            failed.set(true);
            if !err.matches(webkit::DownloadError::CancelledByUser) {
                glib::g_warning!("whatsup-desktop", "a download failed: {}", err.message());
                toasts.add_toast(adw::Toast::new(&format!(
                    "Couldn’t save “{}”",
                    name_of(download)
                )));
            }
        }
    ));
    download.connect_finished(glib::clone!(
        #[weak]
        toasts,
        move |download| {
            if !failed.get() {
                announce_saved(download, &toasts);
            }
        }
    ));
}

fn choose_destination(download: &webkit::Download, suggested: &str) {
    let dir = paths::downloads_dir();
    if let Err(err) = std::fs::create_dir_all(&dir) {
        glib::g_warning!("whatsup-desktop", "cannot create the downloads folder: {err}");
    }
    let path = filename::unique_path(&dir, &filename::sanitize(suggested), Path::exists);
    match path.to_str() {
        Some(path) => download.set_destination(path),
        // WebKit takes the destination as UTF-8; a folder it can't name is refused.
        None => download.cancel(),
    }
}

fn announce_saved(download: &webkit::Download, toasts: &adw::ToastOverlay) {
    let Some(destination) = download.destination() else {
        return;
    };
    let file = gio::File::for_path(destination.as_str());
    let toast = adw::Toast::builder()
        .title(format!("Saved “{}”", name_of(download)))
        .button_label("Open")
        .build();
    toast.connect_button_clicked(glib::clone!(
        #[weak]
        toasts,
        move |_| {
            let window = toasts.root().and_downcast::<gtk::Window>();
            gtk::FileLauncher::new(Some(&file)).launch(
                window.as_ref(),
                gio::Cancellable::NONE,
                |result| {
                    if let Err(err) = result {
                        glib::g_warning!("whatsup-desktop", "cannot open a download: {}", err.message());
                    }
                },
            );
        }
    ));
    toasts.add_toast(toast);
}

/// The saved file's name, for messages to the user.
fn name_of(download: &webkit::Download) -> String {
    download
        .destination()
        .and_then(|path| {
            Path::new(path.as_str())
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "file".to_owned())
}
