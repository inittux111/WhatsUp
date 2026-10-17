//! Where WhatsUp keeps its files, created private (0700) to the user.

use std::fs::{self, DirBuilder};
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use gtk::glib;

const DIR_NAME: &str = "whatsup-desktop";

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

/// The folder holding the user's own stylesheets for WhatsApp Web.
pub fn css_dir() -> PathBuf {
    glib::user_config_dir().join(DIR_NAME).join("css")
}

fn account_path(base: &Path, account: &str) -> PathBuf {
    base.join(DIR_NAME).join("accounts").join(account)
}

/// Creates `dir` and its parents as 0700, and tightens an existing `dir` to 0700.
fn ensure_private(dir: &Path) -> io::Result<()> {
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
    fn ensure_private_creates_and_tightens() {
        let root = std::env::temp_dir().join(format!("whatsup-desktop-test-{}", std::process::id()));
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
