//! The web view that runs WhatsApp Web, locked down to WhatsApp's own site.

mod permissions;
mod policy;
mod style;

use gtk::prelude::*;
use gtk::{gio, glib};
use webkit::prelude::*;

use crate::paths::AccountDirs;

pub const WHATSAPP_URL: &str = "https://web.whatsapp.com/";

/// Builds a web view for one account and starts loading WhatsApp Web.
pub fn build(dirs: &AccountDirs) -> webkit::WebView {
    let session = webkit::NetworkSession::new(dirs.data.to_str(), dirs.cache.to_str());
    let content = webkit::UserContentManager::new();
    style::load(&content);

    let view = webkit::WebView::builder()
        .network_session(&session)
        .user_content_manager(&content)
        .settings(&settings())
        .hexpand(true)
        .vexpand(true)
        .build();

    view.connect_decide_policy(|view, decision, kind| {
        policy::decide(view, decision, kind);
        true
    });
    view.connect_create(|view, action| {
        if let Some(uri) = action.request().and_then(|r| r.uri()) {
            open_externally(view, &uri);
        }
        None
    });
    view.connect_permission_request(permissions::decide);

    view.load_uri(WHATSAPP_URL);
    view
}

fn settings() -> webkit::Settings {
    let settings = webkit::Settings::new();
    settings.set_enable_developer_extras(cfg!(debug_assertions));
    settings.set_enable_media_stream(true);
    settings.set_enable_webrtc(true);
    settings.set_enable_back_forward_navigation_gestures(false);
    settings.set_javascript_can_open_windows_automatically(false);
    settings
}

/// Opens `uri` in the user's default app for it, such as their browser.
pub fn open_externally(view: &webkit::WebView, uri: &str) {
    if !policy::is_openable_externally(uri) {
        return;
    }
    let window = view.root().and_downcast::<gtk::Window>();
    gtk::UriLauncher::new(uri).launch(window.as_ref(), gio::Cancellable::NONE, |result| {
        if let Err(err) = result {
            glib::g_warning!("whatsup-desktop", "cannot open a link: {}", err.message());
        }
    });
}
