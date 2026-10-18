//! The Accounts page: add, rename and remove WhatsApp accounts. Removing
//! one deletes its login data, so it asks first.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use super::{group, new_page};
use crate::settings::Settings;
use crate::settings::accounts::{self, Account};

pub fn page(settings: &Settings) -> adw::PreferencesPage {
    let page = new_page("Accounts", "system-users-symbolic");
    let list = group("Accounts");
    list.set_description(Some(
        "Each account has its own login, kept separately on this computer.",
    ));
    page.add(&list);

    let rows: Rc<RefCell<Vec<adw::EntryRow>>> = Rc::default();
    fill(&list, &rows, settings);

    let add = adw::ButtonRow::builder()
        .title("Add Account")
        .start_icon_name("list-add-symbolic")
        .build();
    add.connect_activated(glib::clone!(
        #[weak]
        list,
        #[strong]
        rows,
        #[strong]
        settings,
        move |_| {
            settings.update(|v| {
                let account = Account {
                    id: accounts::new_id(&v.accounts),
                    name: accounts::new_name(&v.accounts),
                };
                v.active_account = account.id.clone();
                v.accounts.push(account);
            });
            fill(&list, &rows, &settings);
        }
    ));
    let actions = group("");
    actions.set_description(Some(
        "A new account opens with a QR code: scan it with WhatsApp on the phone to link it.",
    ));
    actions.add(&add);
    page.add(&actions);
    page
}

/// Rebuilds the list of accounts from the settings.
fn fill(list: &adw::PreferencesGroup, rows: &Rc<RefCell<Vec<adw::EntryRow>>>, settings: &Settings) {
    for row in rows.borrow_mut().drain(..) {
        list.remove(&row);
    }
    let values = settings.get();
    let only_one = values.accounts.len() == 1;
    for account in values.accounts {
        let row = account_row(list, rows, settings, &account, only_one);
        list.add(&row);
        rows.borrow_mut().push(row);
    }
}

fn account_row(
    list: &adw::PreferencesGroup,
    rows: &Rc<RefCell<Vec<adw::EntryRow>>>,
    settings: &Settings,
    account: &Account,
    only_one: bool,
) -> adw::EntryRow {
    let row = adw::EntryRow::builder()
        .title("Name")
        .text(account.name.as_str())
        .show_apply_button(true)
        .build();
    let id = account.id.clone();
    row.connect_apply(glib::clone!(
        #[strong]
        settings,
        #[strong]
        id,
        move |row| {
            let name = accounts::clean_name(&row.text());
            if name.is_empty() {
                // An account needs a name: show the one it keeps.
                let saved = settings.get().accounts.into_iter().find(|a| a.id == id);
                if let Some(account) = saved {
                    row.set_text(&account.name);
                }
                return;
            }
            row.set_text(&name);
            settings.update(|v| {
                if let Some(account) = v.accounts.iter_mut().find(|a| a.id == id) {
                    account.name = name.clone();
                }
            });
        }
    ));

    let remove = gtk::Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text(if only_one {
            "The Only Account Can’t Be Removed"
        } else {
            "Remove Account"
        })
        .valign(gtk::Align::Center)
        .sensitive(!only_one)
        .css_classes(["flat"])
        .build();
    // Weak: the rows own this button, so a strong reference would keep
    // them, and the whole list, alive after the dialog closes.
    remove.connect_clicked(glib::clone!(
        #[weak]
        list,
        #[weak]
        rows,
        #[strong]
        settings,
        move |button| confirm_remove(button, &list, &rows, &settings, &id)
    ));
    row.add_suffix(&remove);
    row
}

fn confirm_remove(
    anchor: &gtk::Button,
    list: &adw::PreferencesGroup,
    rows: &Rc<RefCell<Vec<adw::EntryRow>>>,
    settings: &Settings,
    id: &str,
) {
    let name = settings
        .get()
        .accounts
        .into_iter()
        .find(|a| a.id == id)
        .map(|a| a.name)
        .unwrap_or_default();
    let dialog = adw::AlertDialog::builder()
        .heading(format!("Remove “{name}”?"))
        .body(
            "WhatsUp logs out of this account and deletes its data from this computer. \
             The chats stay on the phone, and the account can be linked again later.",
        )
        .build();
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("remove", "Remove");
    dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");
    let id = id.to_owned();
    dialog.connect_response(
        Some("remove"),
        glib::clone!(
            #[weak]
            list,
            #[strong]
            rows,
            #[strong]
            settings,
            move |_, _| {
                settings.update(|v| {
                    if v.accounts.len() > 1 {
                        v.accounts.retain(|a| a.id != id);
                    }
                    if v.accounts.iter().all(|a| a.id != v.active_account) {
                        v.active_account = v.accounts[0].id.clone();
                    }
                });
                fill(&list, &rows, &settings);
            }
        ),
    );
    dialog.present(Some(anchor));
}
