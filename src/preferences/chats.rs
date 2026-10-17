//! The Chats page: zoom, spelling and downloads.

use adw::prelude::*;
use gtk::glib;

use super::{depends_on, group, languages, new_page, switch};
use crate::settings::{Settings, Values, ZOOM_MAX, ZOOM_MIN};

pub fn page(
    window: &adw::ApplicationWindow,
    settings: &Settings,
    values: &Values,
) -> adw::PreferencesPage {
    let page = new_page("Chats", "chat-message-new-symbolic");
    page.add(&zoom(settings, values));
    page.add(&spelling(settings, values));
    page.add(&super::downloads::group(window, settings, values));
    page.add(&super::custom::group_for(settings, values));
    page
}

fn zoom(settings: &Settings, values: &Values) -> adw::PreferencesGroup {
    let group = group("Zoom");
    let zoom = adw::SpinRow::with_range(ZOOM_MIN * 100.0, ZOOM_MAX * 100.0, 10.0);
    zoom.set_title("Zoom");
    zoom.set_subtitle("Size of text and pictures, in percent");
    zoom.set_value((values.zoom * 100.0).round());
    zoom.connect_value_notify(glib::clone!(
        #[strong]
        settings,
        move |row| settings.update(|v| v.zoom = row.value() / 100.0)
    ));
    group.add(&zoom);
    group.add(&switch(
        "Show Zoom in Menu",
        "Zoom Out, Reset Zoom and Zoom In in the main menu",
        values.zoom_in_menu,
        settings,
        |v, on| v.zoom_in_menu = on,
    ));
    group
}

fn spelling(settings: &Settings, values: &Values) -> adw::PreferencesGroup {
    let group = group("Spelling");
    let check = switch(
        "Check Spelling",
        "Underline misspelled words while typing",
        values.spell_check,
        settings,
        |v, on| v.spell_check = on,
    );
    let choose = adw::ExpanderRow::builder()
        .title("Spelling Languages")
        .build();
    show_languages(&choose, &values.spell_languages);
    depends_on(&choose, &check);
    glib::spawn_future_local(glib::clone!(
        #[weak]
        choose,
        #[strong]
        settings,
        async move { fill_languages(&choose, &settings, languages::installed().await) }
    ));
    group.add(&check);
    group.add(&choose);
    group
}

/// One check box per installed dictionary; none checked follows the system.
fn fill_languages(choose: &adw::ExpanderRow, settings: &Settings, installed: Vec<String>) {
    if installed.is_empty() {
        choose.set_subtitle("No dictionaries installed");
        return;
    }
    let chosen = settings.get().spell_languages;
    for code in installed {
        let check = gtk::CheckButton::builder()
            .active(chosen.contains(&code))
            .valign(gtk::Align::Center)
            .build();
        let row = adw::ActionRow::builder()
            .title(languages::display_name(&code))
            .activatable_widget(&check)
            .build();
        row.add_prefix(&check);
        check.connect_toggled(glib::clone!(
            #[weak]
            choose,
            #[strong]
            settings,
            move |check| {
                settings.update(|v| {
                    v.spell_languages.retain(|c| *c != code);
                    if check.is_active() {
                        v.spell_languages.push(code.clone());
                    }
                });
                show_languages(&choose, &settings.get().spell_languages);
            }
        ));
        choose.add_row(&row);
    }
}

fn show_languages(choose: &adw::ExpanderRow, chosen: &[String]) {
    let subtitle = if chosen.is_empty() {
        "Same as the system".to_owned()
    } else {
        let names: Vec<String> = chosen.iter().map(|c| languages::display_name(c)).collect();
        names.join(", ")
    };
    choose.set_subtitle(&glib::markup_escape_text(&subtitle));
}
