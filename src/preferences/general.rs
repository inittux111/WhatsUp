//! The General page: the light or dark style, running in the background,
//! the tray icon and notifications.

use adw::prelude::*;

use super::{depends_on, group, new_page, switch};
use crate::settings::{Settings, Style, Values};

pub fn page(settings: &Settings, values: &Values) -> adw::PreferencesPage {
    let page = new_page("General", "preferences-system-symbolic");
    page.add(&appearance(settings, values));
    page.add(&background(settings, values));
    page.add(&tray(settings, values));
    page.add(&notifications(settings, values));
    page
}

fn appearance(settings: &Settings, values: &Values) -> adw::PreferencesGroup {
    let group = group("Appearance");
    let style = adw::ComboRow::builder()
        .title("Style")
        .subtitle("Light or dark, for WhatsUp and WhatsApp")
        .model(&gtk::StringList::new(&["Follow System", "Light", "Dark"]))
        .build();
    let index = Style::ALL.iter().position(|s| *s == values.style);
    style.set_selected(index.unwrap_or_default() as u32);
    let settings = settings.clone();
    style.connect_selected_notify(move |row| {
        if let Some(&chosen) = Style::ALL.get(row.selected() as usize) {
            settings.update(|v| v.style = chosen);
        }
    });
    group.add(&style);
    group
}

fn background(settings: &Settings, values: &Values) -> adw::PreferencesGroup {
    let group = group("Background");
    group.add(&switch(
        "Run in Background",
        "Keep WhatsUp running when the window is closed",
        values.run_in_background,
        settings,
        |v, on| v.run_in_background = on,
    ));
    let login = switch(
        "Start at Login",
        "Open WhatsUp when you log in",
        values.start_at_login,
        settings,
        |v, on| v.start_at_login = on,
    );
    let hidden = switch(
        "Start in Background",
        "When starting at login, stay in the tray",
        values.start_in_background,
        settings,
        |v, on| v.start_in_background = on,
    );
    depends_on(&hidden, &login);
    group.add(&login);
    group.add(&hidden);
    group
}

fn tray(settings: &Settings, values: &Values) -> adw::PreferencesGroup {
    let group = group("Tray and App Icon");
    let show = switch(
        "Show Tray Icon",
        "The icon in the top bar that shows unread chats",
        values.show_tray_icon,
        settings,
        |v, on| v.show_tray_icon = on,
    );
    let colored = switch(
        "Coloured Tray Icon",
        "Green; when off, the icon matches the top bar's other icons",
        values.colored_tray_icon,
        settings,
        |v, on| v.colored_tray_icon = on,
    );
    depends_on(&colored, &show);
    group.add(&show);
    group.add(&colored);
    group.add(&switch(
        "Show Unread Count on App Icon",
        "In docks that show one, such as Dash to Dock",
        values.dock_badge,
        settings,
        |v, on| v.dock_badge = on,
    ));
    group
}

fn notifications(settings: &Settings, values: &Values) -> adw::PreferencesGroup {
    let group = group("Notifications");
    let enabled = switch(
        "Show Notifications",
        "Notify about new messages",
        values.notifications,
        settings,
        |v, on| v.notifications = on,
    );
    let previews = switch(
        "Show Message Previews",
        "Show who wrote and what in notifications",
        values.notification_previews,
        settings,
        |v, on| v.notification_previews = on,
    );
    depends_on(&previews, &enabled);
    group.add(&enabled);
    group.add(&previews);
    group
}
