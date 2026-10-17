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
/// `scripts` runs the user's own scripts in it.
pub fn build(dirs: &AccountDirs, scripts: bool) -> webkit::WebView {
    let session = webkit::NetworkSession::new(dirs.data.to_str(), dirs.cache.to_str());
    let content = webkit::UserContentManager::new();
    style::load(&content, scripts);

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
    if let Some(context) = view.context() {
        permissions::grant_notifications(&context);
    }

    view.load_uri(WHATSAPP_URL);
    view
}

/// Loads the stylesheets (and scripts, if `scripts`) again and reloads the
/// page, so changes to them show.
pub fn reload_customizations(view: &webkit::WebView, scripts: bool) {
    let Some(content) = view.user_content_manager() else {
        return;
    };
    let view = view.clone();
    glib::spawn_future_local(async move {
        style::apply(content, scripts).await;
        view.reload();
    });
}

/// Applies the zoom and spell checking settings now and whenever they change.
pub fn follow_settings(view: &webkit::WebView, settings: &crate::settings::Settings) {
    let apply = glib::clone!(
        #[weak]
        view,
        move |values: &crate::settings::Values| {
            view.set_zoom_level(values.zoom);
            if let Some(context) = view.context() {
                context.set_spell_checking_enabled(values.spell_check);
                let languages = if values.spell_languages.is_empty() {
                    spell_languages(&glib::language_names())
                } else {
                    values.spell_languages.clone()
                };
                let languages: Vec<&str> = languages.iter().map(String::as_str).collect();
                context.set_spell_checking_languages(&languages);
            }
        }
    );
    apply(&settings.get());
    settings.connect_changed(apply);
}

/// The user's languages as spell checking dictionaries ("nl_NL", "en"),
/// from GLib's list, which also holds variants such as "nl_NL.UTF-8" and "C".
fn spell_languages(names: &[glib::GString]) -> Vec<String> {
    let mut languages: Vec<String> = Vec::new();
    for name in names {
        let valid = !name.contains(['.', '@']) && !matches!(name.as_str(), "C" | "POSIX");
        if valid && !languages.iter().any(|l| l == name.as_str()) {
            languages.push(name.to_string());
        }
    }
    languages
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_languages_are_plain_locales() {
        let names: Vec<glib::GString> =
            ["nl_NL.UTF-8", "nl_NL", "nl.UTF-8", "nl", "en_US@euro", "C"]
                .into_iter()
                .map(glib::GString::from)
                .collect();
        assert_eq!(spell_languages(&names), ["nl_NL", "nl"]);
    }
}
