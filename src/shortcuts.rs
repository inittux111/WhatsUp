//! WhatsUp's keyboard shortcuts: one list for the accelerators, the
//! Keyboard Shortcuts dialog, the Settings page and the window's key
//! handling. The user's changes come from the settings.

use std::cell::RefCell;

use adw::prelude::*;
use gtk::glib;

use crate::settings::{Settings, Values};

pub struct Entry {
    pub title: &'static str,
    pub action: &'static str,
    pub defaults: &'static [&'static str],
}

pub const GENERAL: [Entry; 4] = [
    entry("Settings", "win.preferences", &["<Control>comma"]),
    entry(
        "Keyboard Shortcuts",
        "win.shortcuts",
        &["<Control>question"],
    ),
    entry("Close Window", "win.close", &["<Control>w"]),
    entry("Quit", "app.quit", &["<Control>q"]),
];
pub const ACCOUNTS: [Entry; 2] = [
    entry("Next Account", "win.next-account", &["<Control>Page_Down"]),
    entry(
        "Previous Account",
        "win.previous-account",
        &["<Control>Page_Up"],
    ),
];
pub const VIEW: [Entry; 4] = [
    entry(
        "Zoom In",
        "win.zoom-in",
        &["<Control>plus", "<Control>equal"],
    ),
    entry("Zoom Out", "win.zoom-out", &["<Control>minus"]),
    entry("Reset Zoom", "win.zoom-reset", &["<Control>0"]),
    entry("Reload", "win.reload", &["<Control>r", "F5"]),
];

const fn entry(
    title: &'static str,
    action: &'static str,
    defaults: &'static [&'static str],
) -> Entry {
    Entry {
        title,
        action,
        defaults,
    }
}

pub fn all() -> impl Iterator<Item = &'static Entry> {
    GENERAL.iter().chain(ACCOUNTS.iter()).chain(VIEW.iter())
}

/// The keys `entry` uses now: the user's choice, or its defaults.
pub fn keys(values: &Values, entry: &Entry) -> Vec<String> {
    match values.shortcuts.get(entry.action) {
        Some(accel) if accel.is_empty() => Vec::new(),
        Some(accel) => vec![accel.clone()],
        None => entry.defaults.iter().map(|d| (*d).to_owned()).collect(),
    }
}

/// The other shortcut with keys that `same` says match, if any.
pub fn used_by(
    values: &Values,
    action: &str,
    same: impl Fn(&str) -> bool,
) -> Option<&'static Entry> {
    all().find(|e| e.action != action && keys(values, e).iter().any(|k| same(k)))
}

thread_local! {
    /// The main window's capture-phase controller; WhatsUp has one main window.
    static CONTROLLER: RefCell<Option<gtk::ShortcutController>> = const { RefCell::new(None) };
}

/// Applies the shortcuts to `app` and `window`, now and after every change.
pub fn follow(app: &adw::Application, window: &adw::ApplicationWindow, settings: &Settings) {
    let controller = gtk::ShortcutController::new();
    controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    window.add_controller(controller.clone());
    CONTROLLER.with(|c| c.replace(Some(controller)));
    apply(app, &settings.get());
    settings.connect_changed(glib::clone!(
        #[weak]
        app,
        move |values| apply(&app, values)
    ));
}

fn apply(app: &adw::Application, values: &Values) {
    for entry in all() {
        let keys = keys(values, entry);
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        app.set_accels_for_action(entry.action, &keys);
    }
    CONTROLLER.with(|c| {
        if let Some(controller) = c.borrow().as_ref() {
            fill(controller, values);
            controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        }
    });
}

/// Handles WhatsUp's shortcuts before the web page sees the keys, so
/// WhatsApp Web can't swallow them while the user types in a chat.
fn fill(controller: &gtk::ShortcutController, values: &Values) {
    while let Some(shortcut) = controller.item(0).and_downcast::<gtk::Shortcut>() {
        controller.remove_shortcut(&shortcut);
    }
    for entry in all() {
        for accel in keys(values, entry) {
            match gtk::ShortcutTrigger::parse_string(&accel) {
                Some(trigger) => controller.add_shortcut(gtk::Shortcut::new(
                    Some(trigger),
                    Some(gtk::NamedAction::new(entry.action)),
                )),
                None => glib::g_warning!("whatsup-desktop", "invalid shortcut {accel}"),
            }
        }
    }
}

/// Turns every shortcut off while the user records a new one, so the keys
/// they press reach the recorder instead of doing something.
pub fn suspend(app: &adw::Application) {
    for entry in all() {
        app.set_accels_for_action(entry.action, &[]);
    }
    CONTROLLER.with(|c| {
        if let Some(controller) = c.borrow().as_ref() {
            controller.set_propagation_phase(gtk::PropagationPhase::None);
        }
    });
}

/// Turns the shortcuts back on after `suspend`.
pub fn resume(app: &adw::Application, settings: &Settings) {
    apply(app, &settings.get());
}

/// The Keyboard Shortcuts overview. Uses AdwShortcutsDialog (libadwaita 1.8).
#[cfg(feature = "adw-shortcuts")]
pub fn show(window: &adw::ApplicationWindow, values: &Values) {
    let dialog = adw::ShortcutsDialog::new();
    dialog.add(section("General", &GENERAL, values));
    dialog.add(section("Accounts", &ACCOUNTS, values));
    dialog.add(section("View", &VIEW, values));
    dialog.present(Some(window));
}

#[cfg(feature = "adw-shortcuts")]
fn section(title: &str, entries: &[Entry], values: &Values) -> adw::ShortcutsSection {
    let section = adw::ShortcutsSection::new(Some(title));
    for entry in entries {
        if let Some(accel) = keys(values, entry).first() {
            section.add(adw::ShortcutsItem::new(entry.title, accel));
        }
    }
    section
}

/// The same overview for libadwaita 1.7 (Debian stable), which has no
/// AdwShortcutsDialog: a plain dialog of preference groups. GtkShortcutLabel
/// is deprecated in GTK 4.18 but is the only keycap renderer available there.
#[cfg(not(feature = "adw-shortcuts"))]
pub fn show(window: &adw::ApplicationWindow, values: &Values) {
    let page = adw::PreferencesPage::new();
    page.add(&section("General", &GENERAL, values));
    page.add(&section("Accounts", &ACCOUNTS, values));
    page.add(&section("View", &VIEW, values));

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    toolbar.set_content(Some(&page));

    let dialog = adw::Dialog::new();
    dialog.set_title("Keyboard Shortcuts");
    dialog.set_content_width(400);
    dialog.set_content_height(560);
    dialog.set_child(Some(&toolbar));
    dialog.present(Some(window));
}

#[cfg(not(feature = "adw-shortcuts"))]
#[allow(deprecated)] // GtkShortcutLabel: no 1.7 replacement for the keycaps.
fn section(title: &str, entries: &[Entry], values: &Values) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder().title(title).build();
    for entry in entries {
        if let Some(accel) = keys(values, entry).first() {
            let label = gtk::ShortcutLabel::new(accel);
            label.set_valign(gtk::Align::Center);
            let row = adw::ActionRow::builder().title(entry.title).build();
            row.add_suffix(&label);
            group.add(&row);
        }
    }
    group
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::accel;

    #[test]
    fn defaults_are_safe_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for entry in all() {
            assert!(
                !entry.defaults.is_empty(),
                "{} has no shortcut",
                entry.action
            );
            for accel in entry.defaults {
                assert!(accel::is_safe(accel), "{accel} isn't safe");
                assert!(seen.insert(accel.to_lowercase()), "{accel} is used twice");
            }
        }
    }

    #[test]
    fn user_choices_replace_defaults() {
        let mut values = Values::default();
        let zoom_in = &VIEW[0];
        assert_eq!(keys(&values, zoom_in), ["<Control>plus", "<Control>equal"]);
        values
            .shortcuts
            .insert("win.zoom-in".into(), "<Alt>i".into());
        assert_eq!(keys(&values, zoom_in), ["<Alt>i"]);
        values.shortcuts.insert("win.zoom-in".into(), String::new());
        assert!(keys(&values, zoom_in).is_empty());
    }

    #[test]
    fn finds_conflicts() {
        let mut values = Values::default();
        assert_eq!(
            used_by(&values, "win.zoom-in", |k| k == "<Control>r").map(|e| e.action),
            Some("win.reload")
        );
        assert!(used_by(&values, "win.reload", |k| k == "<Control>r").is_none());
        assert!(used_by(&values, "win.zoom-in", |k| k == "<Alt>i").is_none());
        values.shortcuts.insert("win.reload".into(), String::new());
        assert!(used_by(&values, "win.zoom-in", |k| k == "<Control>r").is_none());
    }
}
