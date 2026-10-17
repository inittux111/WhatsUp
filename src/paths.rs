//! Where WhatsUp keeps its files, created private (0700) to the user.

use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use gtk::glib;

const DIR_NAME: &str = "whatsup-desktop";
/// The folders' name in early builds, packaged as `whatsup`.
const OLD_DIR_NAME: &str = "whatsup";
/// What an old folder of ours holds, so a `whatsup` folder of another program
/// is never taken.
const OLD_DIR_MARKERS: [&str; 4] = ["accounts", "settings.ini", "css", "js"];

/// One account's WebKit data and cache folders.
pub struct AccountDirs {
    pub data: PathBuf,
    pub cache: PathBuf,
}

/// The folders for `account`, created private if they don't exist yet.
pub fn account_dirs(account: &str) -> io::Result<AccountDirs> {
    let dirs = AccountDirs {
        data: account_path(&glib::user_data_dir(), account),
        cache: account_path(&glib::user_cache_dir(), account),
    };
    ensure_private(&dirs.data)?;
    ensure_private(&dirs.cache)?;
    Ok(dirs)
}

/// The settings file, in WhatsUp's private config folder.
pub fn settings_file() -> PathBuf {
    glib::user_config_dir().join(DIR_NAME).join("settings.ini")
}

/// Deletes `account`'s data and cache folders, in the background. Only ever
/// for an id that passed `settings::accounts::is_id`, so the paths stay
/// inside WhatsUp's accounts folders.
pub fn remove_account_data(account: &str) {
    if !crate::settings::accounts::is_id(account) {
        return;
    }
    let dirs = [
        account_path(&glib::user_data_dir(), account),
        account_path(&glib::user_cache_dir(), account),
    ];
    gtk::gio::spawn_blocking(move || {
        for dir in dirs {
            if let Err(err) = fs::remove_dir_all(&dir)
                && err.kind() != io::ErrorKind::NotFound
            {
                glib::g_warning!("whatsup-desktop", "cannot remove an account's data: {err}");
            }
        }
    });
}

/// The folder holding the user's own scripts for WhatsApp Web.
pub fn js_dir() -> PathBuf {
    glib::user_config_dir().join(DIR_NAME).join("js")
}

/// The folder holding the user's own stylesheets for WhatsApp Web.
pub fn css_dir() -> PathBuf {
    glib::user_config_dir().join(DIR_NAME).join("css")
}

/// The user's Downloads folder (or home, if they have none set).
pub fn downloads_dir() -> PathBuf {
    glib::user_special_dir(glib::UserDirectory::Downloads).unwrap_or_else(glib::home_dir)
}

/// Where development builds find the app's icons: the source tree's `data/icons`.
/// Installed builds use the system icon theme.
pub fn dev_icon_dir() -> Option<PathBuf> {
    cfg!(debug_assertions).then(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("data/icons"))
}

/// Moves the config, data and cache folders of an early build to their new
/// name, keeping the user's login and settings. Runs once at startup, before
/// anything opens them; each is a single rename within one folder.
pub fn migrate_old_dirs() {
    for base in [
        glib::user_config_dir(),
        glib::user_data_dir(),
        glib::user_cache_dir(),
    ] {
        if let Err(err) = migrate_old_dir(&base) {
            glib::g_warning!("whatsup-desktop", "cannot move an old folder: {err}");
        }
    }
}

fn migrate_old_dir(base: &Path) -> io::Result<()> {
    let old = base.join(OLD_DIR_NAME);
    let new = base.join(DIR_NAME);
    let ours = OLD_DIR_MARKERS.iter().any(|name| old.join(name).exists());
    if !ours || new.exists() {
        return Ok(());
    }
    fs::rename(old, new)
}

fn account_path(base: &Path, account: &str) -> PathBuf {
    base.join(DIR_NAME).join("accounts").join(account)
}

/// Creates `dir` and its parents as 0700, and tightens an existing `dir` to 0700.
pub fn ensure_private(dir: &Path) -> io::Result<()> {
    DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_path_is_per_account() {
        let path = account_path(Path::new("/base"), "default");
        assert_eq!(path, Path::new("/base/whatsup-desktop/accounts/default"));
    }

    #[test]
    fn migrate_moves_only_our_old_folder() {
        let root =
            std::env::temp_dir().join(format!("whatsup-desktop-migrate-{}", std::process::id()));
        let (ours, theirs, both) = (root.join("ours"), root.join("theirs"), root.join("both"));
        fs::create_dir_all(ours.join("whatsup/accounts/default")).unwrap();
        fs::create_dir_all(theirs.join("whatsup/other")).unwrap();
        fs::create_dir_all(both.join("whatsup/accounts")).unwrap();
        fs::create_dir_all(both.join("whatsup-desktop")).unwrap();

        for base in [&ours, &theirs, &both] {
            migrate_old_dir(base).unwrap();
        }
        assert!(ours.join("whatsup-desktop/accounts/default").is_dir());
        assert!(!ours.join("whatsup").exists());
        assert!(theirs.join("whatsup/other").is_dir());
        assert!(!theirs.join("whatsup-desktop").exists());
        assert!(both.join("whatsup/accounts").is_dir());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ensure_private_creates_and_tightens() {
        let root =
            std::env::temp_dir().join(format!("whatsup-desktop-test-{}", std::process::id()));
        let dir = root.join("a/b");
        ensure_private(&dir).unwrap();
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        ensure_private(&dir).unwrap();
        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        fs::remove_dir_all(root).unwrap();
    }
}
