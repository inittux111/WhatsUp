//! The user's settings: loaded once at startup, saved in the background on
//! every change, and announced to the parts of the app that follow them.

pub mod accel;
pub mod accounts;
mod autostart;
mod values;

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

pub use autostart::BACKGROUND_FLAG;
pub use values::{Values, ZOOM_MAX, ZOOM_MIN, clamp_zoom, is_language_code};

use crate::paths;

/// Called with the new values after every change.
type Listener = Rc<dyn Fn(&Values)>;

#[derive(Clone)]
pub struct Settings(Rc<Inner>);

struct Inner {
    values: RefCell<Values>,
    path: PathBuf,
    listeners: RefCell<Vec<Listener>>,
    saving: Cell<bool>,
    dirty: Cell<bool>,
}

impl Settings {
    /// Reads the settings file. It's a few hundred bytes, so this reads it
    /// directly, once, before the window exists.
    pub fn load() -> Self {
        let path = paths::settings_file();
        let values = std::fs::read_to_string(&path)
            .map(|data| Values::parse(&data))
            .unwrap_or_default();
        Self(Rc::new(Inner {
            values: RefCell::new(values),
            path,
            listeners: RefCell::default(),
            saving: Cell::new(false),
            dirty: Cell::new(false),
        }))
    }

    pub fn get(&self) -> Values {
        self.0.values.borrow().clone()
    }

    /// Changes the settings with `change`; if anything changed, saves them
    /// and tells every listener.
    pub fn update(&self, change: impl FnOnce(&mut Values)) {
        let (before, after) = {
            let mut values = self.0.values.borrow_mut();
            let before = values.clone();
            change(&mut values);
            values.zoom = clamp_zoom(values.zoom);
            (before, values.clone())
        };
        if before == after {
            return;
        }
        if (before.start_at_login, before.start_in_background)
            != (after.start_at_login, after.start_in_background)
        {
            autostart::apply(after.start_at_login, after.start_in_background);
        }
        self.save();
        let listeners = self.0.listeners.borrow().clone();
        for listener in listeners {
            listener(&after);
        }
    }

    /// Writes the settings right away, for when the app is about to quit and
    /// a background save wouldn't finish. The file is a few hundred bytes.
    pub fn save_now(&self) {
        let path = &self.0.path;
        let result = path
            .parent()
            .map_or(Ok(()), paths::ensure_private)
            .and_then(|()| write_private(path, &self.get().to_data()));
        if let Err(err) = result {
            glib::g_warning!("whatsup-desktop", "cannot save the settings: {err}");
        }
    }

    /// Calls `f` with the new values after every change.
    pub fn connect_changed(&self, f: impl Fn(&Values) + 'static) {
        self.0.listeners.borrow_mut().push(Rc::new(f));
    }

    /// Writes the file in the background; changes made meanwhile are written
    /// right after, so the file always ends up with the latest values.
    fn save(&self) {
        if self.0.saving.replace(true) {
            self.0.dirty.set(true);
            return;
        }
        let settings = self.clone();
        glib::spawn_future_local(async move {
            loop {
                settings.0.dirty.set(false);
                let data = settings.get().to_data();
                if let Err(err) = write(&settings.0.path, data).await {
                    glib::g_warning!("whatsup-desktop", "cannot save the settings: {}", err.message());
                }
                if !settings.0.dirty.get() {
                    break;
                }
            }
            settings.0.saving.set(false);
        });
    }
}

fn write_private(path: &std::path::Path, data: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let tmp = path.with_extension("ini.tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&tmp)?;
    file.write_all(data.as_bytes())?;
    file.sync_all()?;
    std::fs::rename(tmp, path)
}

async fn write(path: &std::path::Path, data: String) -> Result<(), glib::Error> {
    if let Some(dir) = path.parent() {
        paths::ensure_private(dir)
            .map_err(|e| glib::Error::new(gio::IOErrorEnum::Failed, &e.to_string()))?;
    }
    gio::File::for_path(path)
        .replace_contents_future(
            data.into_bytes(),
            None,
            false,
            gio::FileCreateFlags::PRIVATE,
        )
        .await
        .map(|_| ())
        .map_err(|(_, err)| err)
}
