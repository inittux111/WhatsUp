//! The system theme's colours for WhatsApp Web. GTK has no call that returns
//! a theme colour, so invisible probe widgets each take one of libadwaita's
//! colour variables as their text colour, and WhatsUp reads that back. This
//! sees whatever the active theme sets, Adwaita's or a custom one's such as
//! Abyss, and is read again whenever light/dark or the accent colour changes.

mod css;

use std::time::Duration;

use adw::prelude::*;
use gtk::{gdk, glib};

use css::{Color, NAMES, Palette};

const PROBE_CLASS: &str = "whatsup-desktop-probe";

/// Builds the probes; put the returned widget anywhere in the window. Calls
/// `on_change` with the stylesheet for WhatsApp Web now and after every
/// theme change.
pub fn follow(on_change: impl Fn(String) + 'static) -> gtk::Widget {
    add_probe_styles();
    let probes = gtk::Box::builder()
        .can_target(false)
        .can_focus(false)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .opacity(0.0)
        .build();
    for name in NAMES {
        probes.append(&gtk::Box::builder().css_classes([probe_class(name)]).build());
    }

    let on_change = std::rc::Rc::new(on_change);
    // Styles update on the next frame, so read a moment after a change.
    let read_soon = glib::clone!(
        #[weak]
        probes,
        #[strong]
        on_change,
        move || {
            glib::timeout_add_local_once(
                Duration::from_millis(150),
                glib::clone!(
                    #[weak]
                    probes,
                    #[strong]
                    on_change,
                    move || on_change(css::render(&read(&probes)))
                ),
            );
        }
    );
    let style = adw::StyleManager::default();
    let changed = read_soon.clone();
    style.connect_dark_notify(move |_| changed());
    let changed = read_soon.clone();
    style.connect_accent_color_notify(move |_| changed());
    let changed = read_soon.clone();
    style.connect_high_contrast_notify(move |_| changed());
    probes.connect_map(move |_| read_soon());
    probes.upcast()
}

/// The palette, from each probe's colour in the order of `NAMES`.
fn read(probes: &gtk::Box) -> Palette {
    let mut colors = [Color::WHITE; 13];
    let mut child = probes.first_child();
    for slot in &mut colors {
        let Some(probe) = child else {
            break;
        };
        let rgba = probe.color();
        *slot = Color {
            r: f64::from(rgba.red()),
            g: f64::from(rgba.green()),
            b: f64::from(rgba.blue()),
            a: f64::from(rgba.alpha()),
        };
        child = probe.next_sibling();
    }
    Palette::from_colors(colors)
}

fn probe_class(name: &str) -> String {
    format!("{PROBE_CLASS}-{name}")
}

/// One rule per probe: its text colour is the theme's variable.
fn add_probe_styles() {
    let Some(display) = gdk::Display::default() else {
        return;
    };
    let rules: String = NAMES
        .iter()
        .map(|name| format!(".{} {{ color: var(--{name}-color); }}\n", probe_class(name)))
        .collect();
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&rules);
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
