//! Safe names for downloaded files. The suggested name comes from the web
//! content, so it's cleaned before it touches the file system: no folders,
//! no hidden files, no control characters, and never an existing file.

use std::path::{Path, PathBuf};

const FALLBACK: &str = "download";
const MAX_BYTES: usize = 200;

/// `suggested` as a plain file name, safe to create in the downloads folder.
pub fn sanitize(suggested: &str) -> String {
    // Only the last path component counts, whichever separator the page used.
    let last = suggested.rsplit(['/', '\\']).next().unwrap_or_default();
    let cleaned: String = last
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim().trim_start_matches('.').trim();
    let name = truncate(trimmed, MAX_BYTES);
    if name.is_empty() {
        FALLBACK.to_owned()
    } else {
        name.to_owned()
    }
}

/// The first of `name`, `name (1)`, `name (2)`, ... in `dir` that `exists` says is free.
pub fn unique_path(dir: &Path, name: &str, exists: impl Fn(&Path) -> bool) -> PathBuf {
    let candidate = dir.join(name);
    if !exists(&candidate) {
        return candidate;
    }
    let (stem, extension) = split_extension(name);
    (1u32..)
        .map(|n| dir.join(format!("{stem} ({n}){extension}")))
        .find(|path| !exists(path))
        .expect("an unused name exists")
}

/// "photo.jpg" → ("photo", ".jpg"); "archive" → ("archive", "").
fn split_extension(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(dot) if dot > 0 => name.split_at(dot),
        _ => (name, ""),
    }
}

/// At most `max` bytes of `s`, cut on a character boundary.
fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_ordinary_names() {
        assert_eq!(sanitize("photo.jpg"), "photo.jpg");
        assert_eq!(sanitize("Factuur maart 2026.pdf"), "Factuur maart 2026.pdf");
        assert_eq!(sanitize("café 😀.png"), "café 😀.png");
    }

    #[test]
    fn strips_folders_and_hidden_names() {
        assert_eq!(sanitize("../../.bashrc"), "bashrc");
        assert_eq!(sanitize("/etc/passwd"), "passwd");
        assert_eq!(sanitize("..\\..\\evil.exe"), "evil.exe");
        assert_eq!(sanitize(".hidden"), "hidden");
        assert_eq!(sanitize(".."), FALLBACK);
        assert_eq!(sanitize(""), FALLBACK);
        assert_eq!(sanitize("dir/"), FALLBACK);
    }

    #[test]
    fn replaces_control_and_reserved_characters() {
        assert_eq!(sanitize("a\nb\0c.txt"), "a_b_c.txt");
        assert_eq!(sanitize("what?.txt"), "what_.txt");
    }

    #[test]
    fn limits_the_length_on_a_char_boundary() {
        let long = "é".repeat(300);
        let name = sanitize(&long);
        assert!(name.len() <= MAX_BYTES);
        assert!(name.chars().all(|c| c == 'é'));
    }

    #[test]
    fn never_reuses_an_existing_file() {
        let taken = [
            PathBuf::from("/d/photo.jpg"),
            PathBuf::from("/d/photo (1).jpg"),
        ];
        let exists = |p: &Path| taken.iter().any(|t| t == p);
        assert_eq!(
            unique_path(Path::new("/d"), "new.jpg", exists),
            Path::new("/d/new.jpg")
        );
        assert_eq!(
            unique_path(Path::new("/d"), "photo.jpg", exists),
            Path::new("/d/photo (2).jpg")
        );
        let only = |p: &Path| p == Path::new("/d/archive");
        assert_eq!(
            unique_path(Path::new("/d"), "archive", only),
            Path::new("/d/archive (1)")
        );
    }
}
