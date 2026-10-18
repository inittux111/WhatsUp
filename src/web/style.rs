//! What WhatsUp adds to WhatsApp Web: its own style fixes (`app.css`) and
//! script fixes (`media.js`, `viewer.js`), the system theme's colours (when the user wants them), the user's stylesheets (every `.css` file in `~/.config/whatsup-desktop/css`, such as
//! Abyss's), and, only when the user turned them on, their scripts (every
//! `.js` file in `~/.config/whatsup-desktop/js`). Files are read in the background.

use std::path::Path;

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use crate::paths;

const APP_CSS: &str = include_str!("app.css");
const MEDIA_JS: &str = include_str!("media.js");
const VIEWER_JS: &str = include_str!("viewer.js");
const WHATSAPP: [&str; 1] = ["https://web.whatsapp.com/*"];

/// Loads everything into `content`, replacing what it held. The files are
/// read first and swapped in at once, so overlapping reloads can't mix.
pub fn load(content: &webkit::UserContentManager, extras: Extras) {
    glib::spawn_future_local(apply(content.clone(), extras));
}

/// What to add besides WhatsUp's own fixes.
#[derive(Clone, Debug, Default)]
pub struct Extras {
    /// Run the user's scripts.
    pub scripts: bool,
    /// The system theme's colours as a stylesheet, if the user wants them.
    pub theme: Option<String>,
}

/// `load`, finishing when everything is in place.
pub async fn apply(content: webkit::UserContentManager, extras: Extras) {
    let css = read_all(&paths::css_dir(), "css").await;
    let js = if extras.scripts {
        read_all(&paths::js_dir(), "js").await
    } else {
        Vec::new()
    };
    content.remove_all_style_sheets();
    content.remove_all_scripts();
    add_style_sheet(&content, APP_CSS);
    if let Some(theme) = &extras.theme {
        add_style_sheet(&content, theme);
    }
    for source in &css {
        add_style_sheet(&content, source);
    }
    // Before WhatsApp's own scripts, so its videos already get the fix.
    add_script(&content, MEDIA_JS, webkit::UserScriptInjectionTime::Start);
    add_script(&content, VIEWER_JS, webkit::UserScriptInjectionTime::Start);
    for source in &js {
        add_script(&content, source, webkit::UserScriptInjectionTime::End);
    }
}

/// The contents of the `.<extension>` files directly in `dir`, in name order.
async fn read_all(dir: &Path, extension: &str) -> Vec<String> {
    let folder = gio::File::for_path(dir);
    let mut sources = Vec::new();
    for file in files(&folder, extension).await {
        match file.load_contents_future().await {
            Ok((bytes, _)) => sources.push(String::from_utf8_lossy(&bytes).into_owned()),
            Err(err) => {
                glib::g_warning!(
                    "whatsup-desktop",
                    "cannot read {extension} file: {}",
                    err.message()
                )
            }
        }
    }
    sources
}

/// The `.<extension>` files directly in `folder`, sorted by name so their order is stable.
async fn files(folder: &gio::File, extension: &str) -> Vec<gio::File> {
    let Ok(entries) = folder
        .enumerate_children_future(
            gio::FILE_ATTRIBUTE_STANDARD_NAME,
            gio::FileQueryInfoFlags::NONE,
            glib::Priority::DEFAULT,
        )
        .await
    else {
        return Vec::new();
    };
    let mut names = Vec::new();
    while let Ok(infos) = entries.next_files_future(64, glib::Priority::DEFAULT).await {
        if infos.is_empty() {
            break;
        }
        names.extend(
            infos
                .iter()
                .map(|info| info.name())
                .filter(|n| has_extension(n, extension)),
        );
    }
    names.sort();
    names.iter().map(|name| folder.child(name)).collect()
}

fn has_extension(name: &Path, extension: &str) -> bool {
    name.extension().is_some_and(|ext| ext == extension)
}

fn add_style_sheet(content: &webkit::UserContentManager, source: &str) {
    let sheet = webkit::UserStyleSheet::new(
        source,
        webkit::UserContentInjectedFrames::TopFrame,
        webkit::UserStyleLevel::User,
        &WHATSAPP,
        &[],
    );
    content.add_style_sheet(&sheet);
}

fn add_script(
    content: &webkit::UserContentManager,
    source: &str,
    time: webkit::UserScriptInjectionTime,
) {
    let script = webkit::UserScript::new(
        source,
        webkit::UserContentInjectedFrames::TopFrame,
        time,
        &WHATSAPP,
        &[],
    );
    content.add_script(&script);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_only_matching_files() {
        assert!(has_extension(Path::new("abyss.css"), "css"));
        assert!(!has_extension(Path::new("abyss.css.bak"), "css"));
        assert!(!has_extension(Path::new("css"), "css"));
        assert!(has_extension(Path::new("tweak.js"), "js"));
        assert!(!has_extension(Path::new("tweak.css"), "js"));
    }
}
