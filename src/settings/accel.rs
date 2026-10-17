//! Which keyboard shortcuts the user may set. A shortcut fires while they
//! type in a chat, so it needs Ctrl, Alt or Super, or must be an F-key;
//! a plain letter would fire on every word.

/// Whether `accel` (GTK's form, "<Control>plus") is safe as a shortcut.
pub fn is_safe(accel: &str) -> bool {
    let (modifiers, key) = split(accel);
    if key.is_empty() {
        return false;
    }
    let with_modifier = ["<control>", "<primary>", "<alt>", "<super>", "<meta>"]
        .iter()
        .any(|m| modifiers.to_ascii_lowercase().contains(m));
    with_modifier || is_function_key(key)
}

/// Splits "<Control><Shift>a" into ("<Control><Shift>", "a").
fn split(accel: &str) -> (&str, &str) {
    let end = accel.rfind('>').map_or(0, |i| i + 1);
    accel.split_at(end)
}

/// F1 to F35, as GTK names them.
fn is_function_key(key: &str) -> bool {
    key.strip_prefix('F')
        .and_then(|n| n.parse::<u8>().ok())
        .is_some_and(|n| (1..=35).contains(&n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_a_modifier_or_function_key() {
        assert!(is_safe("<Control>plus"));
        assert!(is_safe("<Control><Shift>question"));
        assert!(is_safe("<Alt>z"));
        assert!(is_safe("<Super>w"));
        assert!(is_safe("F5"));
        assert!(is_safe("F12"));
        assert!(!is_safe("a"));
        assert!(!is_safe("<Shift>a"));
        assert!(!is_safe("space"));
        assert!(!is_safe("F0"));
        assert!(!is_safe("F99"));
        assert!(!is_safe("<Control>"));
        assert!(!is_safe(""));
    }
}
