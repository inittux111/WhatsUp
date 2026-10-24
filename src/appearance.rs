//! The app's light or dark style: the one the user picked in Settings, or
//! the system's. libadwaita restyles the window, and the theme probes
//! (`theme`) pass the new colours on to WhatsApp Web.

use crate::settings::{Settings, Style};

/// Applies the chosen style now and after every change.
pub fn follow(settings: &Settings) {
    apply(settings.get().style);
    settings.connect_changed(|values| apply(values.style));
}

fn apply(style: Style) {
    adw::StyleManager::default().set_color_scheme(match style {
        Style::System => adw::ColorScheme::Default,
        Style::Light => adw::ColorScheme::ForceLight,
        Style::Dark => adw::ColorScheme::ForceDark,
    });
}
