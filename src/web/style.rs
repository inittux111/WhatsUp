//! Stylesheets for WhatsApp Web: the app's own fixes (`app.css`), then the
//! user's, such as Abyss's: every `.css` file in `~/.config/whatsup-desktop/css`,
//! read in the background.

use gtk::gio;
use gtk::glib;
use gtk::prelude::*;

use crate::paths;

const APP_CSS: &str = include_str!("app.css");

pub fn load(content: &webkit::UserContentManager) {
    add_style_sheet(content, APP_CSS);
    load_user_css(content);
}

fn load_user_css(content: &webkit::UserContentManager) {
    let content = content.clone();
    glib::spawn_future_local(async move {
        let folder = gio::File::for_path(paths::css_dir());
        for file in css_files(&folder).await {
            match file.load_contents_future().await {
                Ok((bytes, _)) => add_style_sheet(&content, &String::from_utf8_lossy(&bytes)),
                Err(err) => {
                    glib::g_warning!("whatsup-desktop", "cannot read a stylesheet: {}", err.message())
                }
            }
        }
    });
}

/// The `.css` files directly in `folder`, sorted by name so their order is stable.
async fn css_files(folder: &gio::File) -> Vec<gio::File> {
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
        names.extend(infos.iter().map(|info| info.name()).filter(|n| is_css(n)));
    }
    names.sort();
    names.iter().map(|name| folder.child(name)).collect()
}

fn is_css(name: &std::path::Path) -> bool {
    name.extension().is_some_and(|ext| ext == "css")
}

fn add_style_sheet(content: &webkit::UserContentManager, source: &str) {
    let sheet = webkit::UserStyleSheet::new(
        source,
        webkit::UserContentInjectedFrames::TopFrame,
        webkit::UserStyleLevel::User,
        &["https://web.whatsapp.com/*"],
        &[],
    );
    content.add_style_sheet(&sheet);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn picks_only_css_files() {
        assert!(is_css(Path::new("abyss.css")));
        assert!(!is_css(Path::new("abyss.css.bak")));
        assert!(!is_css(Path::new("css")));
    }
}
