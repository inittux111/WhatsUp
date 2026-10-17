//! The user's settings and how they're stored: a small key file.

use std::collections::BTreeMap;
use std::path::PathBuf;

use gtk::glib;

use super::accel;
use super::accounts::{self, Account};

pub const ZOOM_MIN: f64 = 0.5;
pub const ZOOM_MAX: f64 = 3.0;
const WINDOW_MIN: i32 = 360;
const WINDOW_MAX: i32 = 16384;

#[derive(Clone, Debug, PartialEq)]
pub struct Values {
    /// Closing the window keeps WhatsUp running (it hides to the tray).
    pub run_in_background: bool,
    pub start_at_login: bool,
    /// When started at login, start hidden in the tray.
    pub start_in_background: bool,
    pub show_tray_icon: bool,
    /// The tray icon in green, rather than symbolic in the top bar's colour.
    pub colored_tray_icon: bool,
    /// The unread count on the app's icon, in docks that show one.
    pub dock_badge: bool,
    pub notifications: bool,
    /// Notifications show the sender and message text.
    pub notification_previews: bool,
    pub zoom: f64,
    /// The main menu shows Zoom Out / Reset Zoom / Zoom In.
    pub zoom_in_menu: bool,
    pub spell_check: bool,
    /// Dictionaries such as "nl_NL"; empty follows the system's languages.
    pub spell_languages: Vec<String>,
    /// `None` is the user's Downloads folder.
    pub download_folder: Option<PathBuf>,
    /// Ask where to save each download instead of using the folder.
    pub ask_download_location: bool,
    /// Shortcuts the user changed, by action ("win.zoom-in"); "" turns one off.
    pub shortcuts: BTreeMap<String, String>,
    pub window: WindowSize,
    /// Never empty; the first is shown first on a fresh start.
    pub accounts: Vec<Account>,
    /// The account shown when WhatsUp starts.
    pub active_account: String,
    /// Run the user's own scripts from `~/.config/whatsup-desktop/js` in WhatsApp Web.
    pub custom_scripts: bool,
    /// WhatsApp Web takes its colours from the system theme.
    pub system_colors: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
}

impl Default for Values {
    fn default() -> Self {
        Self {
            run_in_background: true,
            start_at_login: false,
            start_in_background: true,
            show_tray_icon: true,
            colored_tray_icon: true,
            dock_badge: true,
            notifications: true,
            notification_previews: true,
            zoom: 1.0,
            zoom_in_menu: true,
            spell_check: true,
            spell_languages: Vec::new(),
            download_folder: None,
            ask_download_location: false,
            shortcuts: BTreeMap::new(),
            window: WindowSize {
                width: 1100,
                height: 760,
                maximized: false,
            },
            accounts: accounts::defaults(),
            active_account: accounts::FIRST_ID.to_owned(),
            custom_scripts: false,
            system_colors: true,
        }
    }
}

const GENERAL: &str = "General";
const NOTIFICATIONS: &str = "Notifications";
const CHATS: &str = "Chats";
const DOWNLOADS: &str = "Downloads";
const MENU: &str = "Menu";
const SHORTCUTS: &str = "Shortcuts";
const WINDOW: &str = "Window";

impl Values {
    /// Reads `data`; anything missing or invalid keeps its default.
    pub fn parse(data: &str) -> Self {
        let file = glib::KeyFile::new();
        let mut v = Self::default();
        if file.load_from_data(data, glib::KeyFileFlags::NONE).is_err() {
            return v;
        }
        let flag = |group, key, default: bool| file.boolean(group, key).unwrap_or(default);
        v.run_in_background = flag(GENERAL, "run-in-background", v.run_in_background);
        v.start_at_login = flag(GENERAL, "start-at-login", v.start_at_login);
        v.start_in_background = flag(GENERAL, "start-in-background", v.start_in_background);
        v.show_tray_icon = flag(GENERAL, "tray-icon", v.show_tray_icon);
        v.colored_tray_icon = flag(GENERAL, "colored-tray-icon", v.colored_tray_icon);
        v.dock_badge = flag(GENERAL, "dock-badge", v.dock_badge);
        v.notifications = flag(NOTIFICATIONS, "enabled", v.notifications);
        v.notification_previews = flag(NOTIFICATIONS, "previews", v.notification_previews);
        if let Ok(zoom) = file.double(CHATS, "zoom") {
            v.zoom = clamp_zoom(zoom);
        }
        v.zoom_in_menu = flag(MENU, "zoom", v.zoom_in_menu);
        v.spell_check = flag(CHATS, "spell-check", v.spell_check);
        if let Ok(languages) = file.string(CHATS, "spell-languages") {
            v.spell_languages = languages
                .split(';')
                .filter(|l| is_language_code(l))
                .map(str::to_owned)
                .collect();
        }
        v.download_folder = file
            .string(DOWNLOADS, "folder")
            .ok()
            .map(|folder| PathBuf::from(folder.as_str()))
            .filter(|folder| folder.is_absolute());
        v.ask_download_location = flag(DOWNLOADS, "ask", v.ask_download_location);
        for action in file.keys(SHORTCUTS).map(|k| k.to_vec()).unwrap_or_default() {
            if let Ok(accel) = file.string(SHORTCUTS, &action)
                && (accel.is_empty() || accel::is_safe(&accel))
            {
                v.shortcuts.insert(action.to_string(), accel.to_string());
            }
        }
        let size = |key, default| {
            file.integer(WINDOW, key)
                .map_or(default, |n| n.clamp(WINDOW_MIN, WINDOW_MAX))
        };
        v.window = WindowSize {
            width: size("width", v.window.width),
            height: size("height", v.window.height),
            maximized: flag(WINDOW, "maximized", v.window.maximized),
        };
        v.accounts = accounts::read(&file);
        v.active_account = file
            .string(GENERAL, "active-account")
            .map(|id| id.to_string())
            .ok()
            .filter(|id| v.accounts.iter().any(|a| a.id == *id))
            .unwrap_or_else(|| v.accounts[0].id.clone());
        v.custom_scripts = flag(CHATS, "custom-scripts", v.custom_scripts);
        v.system_colors = flag(CHATS, "system-colors", v.system_colors);
        v
    }

    pub fn to_data(&self) -> String {
        let file = glib::KeyFile::new();
        file.set_boolean(GENERAL, "run-in-background", self.run_in_background);
        file.set_boolean(GENERAL, "start-at-login", self.start_at_login);
        file.set_boolean(GENERAL, "start-in-background", self.start_in_background);
        file.set_boolean(GENERAL, "tray-icon", self.show_tray_icon);
        file.set_boolean(GENERAL, "colored-tray-icon", self.colored_tray_icon);
        file.set_boolean(GENERAL, "dock-badge", self.dock_badge);
        file.set_boolean(NOTIFICATIONS, "enabled", self.notifications);
        file.set_boolean(NOTIFICATIONS, "previews", self.notification_previews);
        file.set_double(CHATS, "zoom", self.zoom);
        file.set_boolean(CHATS, "spell-check", self.spell_check);
        // A ";"-separated list; language codes never contain ";".
        file.set_string(CHATS, "spell-languages", &self.spell_languages.join(";"));
        file.set_boolean(MENU, "zoom", self.zoom_in_menu);
        if let Some(folder) = self.download_folder.as_ref().and_then(|f| f.to_str()) {
            file.set_string(DOWNLOADS, "folder", folder);
        }
        file.set_boolean(DOWNLOADS, "ask", self.ask_download_location);
        for (action, accel) in &self.shortcuts {
            file.set_string(SHORTCUTS, action, accel);
        }
        file.set_integer(WINDOW, "width", self.window.width);
        file.set_integer(WINDOW, "height", self.window.height);
        file.set_boolean(WINDOW, "maximized", self.window.maximized);
        accounts::write(&file, &self.accounts);
        file.set_string(GENERAL, "active-account", &self.active_account);
        file.set_boolean(CHATS, "custom-scripts", self.custom_scripts);
        file.set_boolean(CHATS, "system-colors", self.system_colors);
        file.to_data().to_string()
    }
}

/// `zoom` within the range the app offers, to whole percents; not-a-number is 100%.
pub fn clamp_zoom(zoom: f64) -> f64 {
    if zoom.is_nan() {
        return 1.0;
    }
    ((zoom * 100.0).round() / 100.0).clamp(ZOOM_MIN, ZOOM_MAX)
}

/// "nl", "nl_NL", "pt_BR": the form spell checking dictionaries use.
pub fn is_language_code(code: &str) -> bool {
    let mut parts = code.split('_');
    let language = parts.next().unwrap_or_default();
    let region = parts.next();
    parts.next().is_none()
        && (2..=3).contains(&language.len())
        && language.chars().all(|c| c.is_ascii_lowercase())
        && region.is_none_or(|r| {
            (2..=3).contains(&r.len()) && r.chars().all(|c| c.is_ascii_alphanumeric())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let values = Values {
            run_in_background: false,
            start_at_login: true,
            start_in_background: false,
            show_tray_icon: false,
            colored_tray_icon: false,
            dock_badge: false,
            notifications: false,
            notification_previews: false,
            zoom: 1.25,
            zoom_in_menu: false,
            spell_check: false,
            spell_languages: vec!["nl_NL".into(), "en_US".into()],
            download_folder: Some(PathBuf::from("/home/me/Saved Chats")),
            ask_download_location: true,
            shortcuts: BTreeMap::from([
                ("win.zoom-in".into(), "<Alt>i".into()),
                ("win.reload".into(), String::new()),
            ]),
            window: WindowSize {
                width: 900,
                height: 700,
                maximized: true,
            },
            accounts: vec![
                Account {
                    id: "default".into(),
                    name: "Personal".into(),
                },
                Account {
                    id: "account-2".into(),
                    name: "Work".into(),
                },
            ],
            active_account: "account-2".into(),
            custom_scripts: true,
            system_colors: false,
        };
        assert_eq!(Values::parse(&values.to_data()), values);
    }

    #[test]
    fn missing_or_broken_data_keeps_defaults() {
        assert_eq!(Values::parse(""), Values::default());
        assert_eq!(Values::parse("not a key file ["), Values::default());
        let partial = Values::parse("[Chats]\nzoom=1.5\nspell-check=maybe\n");
        assert_eq!(partial.zoom, 1.5);
        assert!(partial.spell_check);
    }

    #[test]
    fn rejects_unsafe_values() {
        assert_eq!(Values::parse("[Chats]\nzoom=40\n").zoom, ZOOM_MAX);
        assert_eq!(Values::parse("[Chats]\nzoom=0\n").zoom, ZOOM_MIN);
        assert_eq!(clamp_zoom(f64::NAN), 1.0);
        assert_eq!(clamp_zoom(1.0 + 0.1), 1.1);
        assert_eq!(clamp_zoom(1.234), 1.23);
        let relative = Values::parse("[Downloads]\nfolder=../elsewhere\n");
        assert_eq!(relative.download_folder, None);
        let shortcut = Values::parse("[Shortcuts]\nwin.reload=r\nwin.zoom-in=<Control>i\n");
        assert_eq!(shortcut.shortcuts.len(), 1);
        let tiny = Values::parse("[Window]\nwidth=1\nheight=999999\n");
        assert_eq!(
            (tiny.window.width, tiny.window.height),
            (WINDOW_MIN, WINDOW_MAX)
        );
        let languages = Values::parse("[Chats]\nspell-languages=nl_NL;../x;en;\n");
        assert_eq!(languages.spell_languages, ["nl_NL", "en"]);
    }

    #[test]
    fn language_codes() {
        assert!(is_language_code("nl"));
        assert!(is_language_code("nl_NL"));
        assert!(is_language_code("pt_BR"));
        assert!(!is_language_code("NL"));
        assert!(!is_language_code("nl_NL.UTF-8"));
        assert!(!is_language_code("../etc"));
        assert!(!is_language_code(""));
    }
}
