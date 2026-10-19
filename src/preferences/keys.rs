//! The Shortcuts page: every shortcut, changeable. Clicking one opens a
//! recorder; a new shortcut must be safe while typing (see
//! `settings::accel`) and not already in use.

use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, glib};

use super::{group, new_page};
use crate::settings::{Settings, accel};
use crate::shortcuts::{self, Entry};

/// The keycap widget: AdwShortcutLabel on libadwaita 1.8, and the
/// (deprecated) GtkShortcutLabel it was forked from on 1.7 (Debian stable).
#[cfg(feature = "adw-shortcuts")]
type KeyLabel = adw::ShortcutLabel;
#[cfg(not(feature = "adw-shortcuts"))]
#[allow(deprecated)] // GtkShortcutLabel: no 1.7 replacement for the keycaps.
type KeyLabel = gtk::ShortcutLabel;

/// One shortcut's row, and the widgets that show its keys.
struct Row {
    entry: &'static Entry,
    row: adw::ActionRow,
    keys: KeyLabel,
    reset: gtk::Button,
}

/// The page's rows. Signal handlers hold them weakly: the rows own the
/// widgets, which own the handlers.
type Rows = Rc<Vec<Row>>;

pub fn page(app: &adw::Application, settings: &Settings) -> adw::PreferencesPage {
    let page = new_page(
        "Shortcuts",
        "preferences-desktop-keyboard-shortcuts-symbolic",
    );
    let mut rows = Vec::new();
    for (title, entries) in [
        ("General", &shortcuts::GENERAL[..]),
        ("Accounts", &shortcuts::ACCOUNTS[..]),
        ("View", &shortcuts::VIEW[..]),
    ] {
        let group = group(title);
        rows.extend(entries.iter().map(|entry| row(&group, entry)));
        page.add(&group);
    }
    let rows: Rows = Rc::new(rows);
    connect(app, settings, &rows);
    refresh(&rows, settings);
    // The page keeps the rows alive; the handlers above only hold them weakly.
    let keep = rows.clone();
    page.connect_destroy(move |_| {
        let _ = &keep;
    });

    let reset_all = gtk::Button::builder()
        .label("Reset All")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    let weak = Rc::downgrade(&rows);
    reset_all.connect_clicked(glib::clone!(
        #[strong]
        settings,
        move |_| {
            settings.update(|v| v.shortcuts.clear());
            if let Some(rows) = weak.upgrade() {
                refresh(&rows, &settings);
            }
        }
    ));
    let footer = group("");
    footer.set_header_suffix(Some(&reset_all));
    footer.set_description(Some(
        "Shortcuts need Ctrl, Alt or Super, or an F-key, so they don’t fire while you type.",
    ));
    page.add(&footer);
    page
}

#[cfg_attr(not(feature = "adw-shortcuts"), allow(deprecated))] // GtkShortcutLabel on 1.7.
fn row(group: &adw::PreferencesGroup, entry: &'static Entry) -> Row {
    let keys = KeyLabel::new("");
    keys.set_disabled_text("Off");
    keys.set_valign(gtk::Align::Center);
    let reset = gtk::Button::builder()
        .icon_name("edit-undo-symbolic")
        .tooltip_text("Reset to Default")
        .valign(gtk::Align::Center)
        .css_classes(["flat"])
        .build();
    let row = adw::ActionRow::builder()
        .title(entry.title)
        .activatable(true)
        .build();
    row.add_suffix(&keys);
    row.add_suffix(&reset);
    group.add(&row);
    Row {
        entry,
        row,
        keys,
        reset,
    }
}

fn connect(app: &adw::Application, settings: &Settings, rows: &Rows) {
    for (index, row) in rows.iter().enumerate() {
        let entry = row.entry;
        let weak = Rc::downgrade(rows);
        row.reset.connect_clicked(glib::clone!(
            #[strong]
            settings,
            move |_| {
                settings.update(|v| {
                    v.shortcuts.remove(entry.action);
                });
                if let Some(rows) = weak.upgrade() {
                    refresh(&rows, &settings);
                }
            }
        ));
        let weak = Rc::downgrade(rows);
        row.row.connect_activated(glib::clone!(
            #[weak]
            app,
            #[strong]
            settings,
            move |row| {
                if let Some(rows) = weak.upgrade() {
                    record(&app, row, &settings, &rows, index);
                }
            }
        ));
    }
}

/// Shows each shortcut's current keys, and a reset button where it differs.
#[cfg_attr(not(feature = "adw-shortcuts"), allow(deprecated))] // set_accelerator on 1.7.
fn refresh(rows: &[Row], settings: &Settings) {
    let values = settings.get();
    for row in rows {
        let keys = shortcuts::keys(&values, row.entry);
        row.keys.set_accelerator(&keys.join(" "));
        row.reset
            .set_visible(values.shortcuts.contains_key(row.entry.action));
    }
}

/// Opens the recorder for `rows[index]`; WhatsUp's shortcuts are off meanwhile.
fn record(
    app: &adw::Application,
    anchor: &adw::ActionRow,
    settings: &Settings,
    rows: &Rows,
    index: usize,
) {
    let entry = rows[index].entry;
    let dialog = adw::AlertDialog::builder()
        .heading(entry.title)
        .body("Press the new shortcut.\nEscape cancels, Backspace turns it off.")
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.set_close_response("cancel");

    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    keys.connect_key_pressed(glib::clone!(
        #[weak]
        dialog,
        #[strong]
        settings,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_, key, _, state| {
            match pressed(key, state) {
                Pressed::Modifier => {}
                Pressed::Cancel => {
                    dialog.close();
                }
                Pressed::TurnOff => {
                    settings.update(|v| {
                        v.shortcuts.insert(entry.action.to_owned(), String::new());
                    });
                    dialog.close();
                }
                Pressed::Keys(accel) => match check(&settings, entry, &accel) {
                    Ok(()) => {
                        settings.update(|v| {
                            v.shortcuts.insert(entry.action.to_owned(), accel);
                        });
                        dialog.close();
                    }
                    Err(problem) => dialog.set_body(&problem),
                },
            }
            glib::Propagation::Stop
        }
    ));
    dialog.add_controller(keys);

    shortcuts::suspend(app);
    dialog.connect_closed(glib::clone!(
        #[weak]
        app,
        #[strong]
        settings,
        #[strong]
        rows,
        move |_| {
            shortcuts::resume(&app, &settings);
            refresh(&rows, &settings);
        }
    ));
    dialog.present(Some(anchor));
}

enum Pressed {
    Modifier,
    Cancel,
    TurnOff,
    Keys(String),
}

fn pressed(key: gdk::Key, state: gdk::ModifierType) -> Pressed {
    let state = state & gtk::accelerator_get_default_mod_mask();
    if is_modifier(key) {
        return Pressed::Modifier;
    }
    match key {
        gdk::Key::Escape if state.is_empty() => Pressed::Cancel,
        gdk::Key::BackSpace if state.is_empty() => Pressed::TurnOff,
        _ => Pressed::Keys(gtk::accelerator_name(key.to_lower(), state).to_string()),
    }
}

fn is_modifier(key: gdk::Key) -> bool {
    key.name().is_some_and(|name| {
        let name = name.as_str();
        name.ends_with("_L")
            || name.ends_with("_R")
            || matches!(
                name,
                "ISO_Level3_Shift" | "ISO_Level5_Shift" | "Caps_Lock" | "Num_Lock"
            )
    })
}

/// Why `accel` can't be used for `entry`, if it can't.
fn check(settings: &Settings, entry: &Entry, accel: &str) -> Result<(), String> {
    if !accel::is_safe(accel) {
        return Err("Use Ctrl, Alt or Super with a key, or an F-key.\nEscape cancels.".to_owned());
    }
    let wanted = gtk::accelerator_parse(accel);
    let same = |other: &str| wanted.is_some() && gtk::accelerator_parse(other) == wanted;
    match shortcuts::used_by(&settings.get(), entry.action, same) {
        Some(other) => Err(format!(
            "That’s already the shortcut for “{}”.\nPress another, or Escape to cancel.",
            other.title
        )),
        None => Ok(()),
    }
}
