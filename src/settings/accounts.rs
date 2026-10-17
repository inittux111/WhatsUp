//! The user's WhatsApp accounts: an id, which names the account's private
//! data folder, and a name the user chooses.

use gtk::glib;

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub id: String,
    pub name: String,
}

/// The id of the first account, which earlier versions used for the only one.
pub const FIRST_ID: &str = "default";
const MAX_NAME: usize = 40;

/// The accounts on a fresh install: one, named after WhatsApp.
pub fn defaults() -> Vec<Account> {
    vec![Account {
        id: FIRST_ID.to_owned(),
        name: "WhatsApp".to_owned(),
    }]
}

/// Whether `id` is safe as a folder name: lowercase letters, digits and
/// dashes only, so it can never point outside the accounts folder.
pub fn is_id(id: &str) -> bool {
    (1..=40).contains(&id.len())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// An id that none of `accounts` uses: "account-2", "account-3", ...
pub fn new_id(accounts: &[Account]) -> String {
    (2u32..)
        .map(|n| format!("account-{n}"))
        .find(|id| accounts.iter().all(|a| a.id != *id))
        .expect("an unused id exists")
}

/// A name for a new account, such as "Account 2".
pub fn new_name(accounts: &[Account]) -> String {
    (accounts.len() + 1..)
        .map(|n| format!("Account {n}"))
        .find(|name| accounts.iter().all(|a| a.name != *name))
        .expect("an unused name exists")
}

/// `name` cleaned up for display: one line, trimmed, not too long.
pub fn clean_name(name: &str) -> String {
    let one_line: String = name
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    one_line.trim().chars().take(MAX_NAME).collect()
}

const GROUP: &str = "Accounts";

pub fn read(file: &glib::KeyFile) -> Vec<Account> {
    let Ok(ids) = file.string(GROUP, "ids") else {
        return defaults();
    };
    let mut accounts: Vec<Account> = Vec::new();
    for id in ids.split(';').filter(|id| is_id(id)) {
        if accounts.iter().any(|a| a.id == id) {
            continue;
        }
        let name = file
            .string(&group_of(id), "name")
            .map(|n| clean_name(&n))
            .ok()
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| new_name(&accounts));
        accounts.push(Account {
            id: id.to_owned(),
            name,
        });
    }
    if accounts.is_empty() {
        defaults()
    } else {
        accounts
    }
}

pub fn write(file: &glib::KeyFile, accounts: &[Account]) {
    let ids: Vec<&str> = accounts.iter().map(|a| a.id.as_str()).collect();
    file.set_string(GROUP, "ids", &ids.join(";"));
    for account in accounts {
        file.set_string(&group_of(&account.id), "name", &account.name);
    }
}

fn group_of(id: &str) -> String {
    format!("Account {id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, name: &str) -> Account {
        Account {
            id: id.into(),
            name: name.into(),
        }
    }

    #[test]
    fn ids_are_safe_folder_names() {
        assert!(is_id("default"));
        assert!(is_id("account-2"));
        assert!(!is_id(""));
        assert!(!is_id("../x"));
        assert!(!is_id("a/b"));
        assert!(!is_id("Work"));
        assert!(!is_id(&"a".repeat(41)));
    }

    #[test]
    fn new_ids_and_names_are_unused() {
        let accounts = vec![
            account("default", "WhatsApp"),
            account("account-2", "Account 2"),
        ];
        assert_eq!(new_id(&accounts), "account-3");
        assert_eq!(new_name(&accounts), "Account 3");
        assert_eq!(new_id(&[account("account-3", "x")]), "account-2");
    }

    #[test]
    fn names_are_cleaned() {
        assert_eq!(clean_name("  Work\nphone "), "Work phone");
        assert_eq!(clean_name(&"x".repeat(100)).len(), MAX_NAME);
    }

    #[test]
    fn round_trips_and_rejects_bad_ids() {
        let file = glib::KeyFile::new();
        let accounts = vec![account("default", "Personal"), account("account-2", "Work")];
        write(&file, &accounts);
        assert_eq!(read(&file), accounts);

        let bad = glib::KeyFile::new();
        bad.set_string(GROUP, "ids", "../etc;default;default;Work");
        assert_eq!(
            read(&bad).iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
            ["default"]
        );
        assert_eq!(read(&glib::KeyFile::new()), defaults());
    }
}
