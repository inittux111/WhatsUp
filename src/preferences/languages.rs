//! The spelling dictionaries installed on the system, and readable names
//! for them ("nl_NL" → "Dutch (Netherlands)").

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::settings::is_language_code;

/// Where Hunspell dictionaries live; WebKit checks spelling through Enchant,
/// which uses them.
const DICTIONARY_DIRS: [&str; 2] = ["/usr/share/hunspell", "/usr/share/myspell"];

/// The installed dictionaries, sorted by name, read in the background.
pub async fn installed() -> Vec<String> {
    let mut codes = Vec::new();
    for dir in DICTIONARY_DIRS {
        let folder = gio::File::for_path(dir);
        let Ok(entries) = folder
            .enumerate_children_future(
                gio::FILE_ATTRIBUTE_STANDARD_NAME,
                gio::FileQueryInfoFlags::NONE,
                glib::Priority::DEFAULT,
            )
            .await
        else {
            continue;
        };
        while let Ok(infos) = entries.next_files_future(64, glib::Priority::DEFAULT).await {
            if infos.is_empty() {
                break;
            }
            for info in infos {
                let name = info.name();
                let code = name.to_str().and_then(|n| n.strip_suffix(".dic"));
                if let Some(code) = code.filter(|c| is_language_code(c))
                    && !codes.iter().any(|known| known == code)
                {
                    codes.push(code.to_owned());
                }
            }
        }
    }
    codes.sort_by_key(|code| display_name(code));
    codes
}

/// A readable name for a dictionary code; unknown parts stay as codes.
pub fn display_name(code: &str) -> String {
    let (language, region) = code.split_once('_').unwrap_or((code, ""));
    let language = LANGUAGES
        .iter()
        .find(|(c, _)| *c == language)
        .map_or(language, |(_, name)| name);
    if region.is_empty() {
        return language.to_owned();
    }
    let region = REGIONS
        .iter()
        .find(|(c, _)| *c == region)
        .map_or(region, |(_, name)| name);
    format!("{language} ({region})")
}

const LANGUAGES: [(&str, &str); 30] = [
    ("ar", "Arabic"),
    ("bg", "Bulgarian"),
    ("ca", "Catalan"),
    ("cs", "Czech"),
    ("da", "Danish"),
    ("de", "German"),
    ("el", "Greek"),
    ("en", "English"),
    ("es", "Spanish"),
    ("et", "Estonian"),
    ("fi", "Finnish"),
    ("fr", "French"),
    ("fy", "Frisian"),
    ("he", "Hebrew"),
    ("hr", "Croatian"),
    ("hu", "Hungarian"),
    ("id", "Indonesian"),
    ("it", "Italian"),
    ("lt", "Lithuanian"),
    ("lv", "Latvian"),
    ("nb", "Norwegian Bokmål"),
    ("nl", "Dutch"),
    ("pl", "Polish"),
    ("pt", "Portuguese"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("sk", "Slovak"),
    ("sv", "Swedish"),
    ("tr", "Turkish"),
    ("uk", "Ukrainian"),
];

const REGIONS: [(&str, &str); 24] = [
    ("AR", "Argentina"),
    ("AT", "Austria"),
    ("AU", "Australia"),
    ("BE", "Belgium"),
    ("BR", "Brazil"),
    ("CA", "Canada"),
    ("CH", "Switzerland"),
    ("DE", "Germany"),
    ("DK", "Denmark"),
    ("ES", "Spain"),
    ("FR", "France"),
    ("GB", "United Kingdom"),
    ("IE", "Ireland"),
    ("IN", "India"),
    ("IT", "Italy"),
    ("LU", "Luxembourg"),
    ("MX", "Mexico"),
    ("NL", "Netherlands"),
    ("NZ", "New Zealand"),
    ("PT", "Portugal"),
    ("SR", "Suriname"),
    ("SE", "Sweden"),
    ("US", "United States"),
    ("ZA", "South Africa"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_known_and_unknown_codes() {
        assert_eq!(display_name("nl_NL"), "Dutch (Netherlands)");
        assert_eq!(display_name("en_US"), "English (United States)");
        assert_eq!(display_name("de"), "German");
        assert_eq!(display_name("xx_YY"), "xx (YY)");
    }
}
