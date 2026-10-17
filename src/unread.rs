//! The unread count, read from WhatsApp Web's page title: "(3) WhatsApp".

/// The number of unread chats in `title`, or 0 when it shows none.
pub fn count_from_title(title: &str) -> u32 {
    let Some(rest) = title.trim_start().strip_prefix('(') else {
        return 0;
    };
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let after = &rest[digits.len()..];
    if digits.is_empty() || !(after.starts_with(')') || after.starts_with("+)")) {
        return 0;
    }
    digits.parse().unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_count() {
        assert_eq!(count_from_title("(3) WhatsApp"), 3);
        assert_eq!(count_from_title("(120) WhatsApp"), 120);
        assert_eq!(count_from_title("(99+) WhatsApp"), 99);
        assert_eq!(count_from_title("(99999999999) WhatsApp"), u32::MAX);
    }

    #[test]
    fn no_count_is_zero() {
        assert_eq!(count_from_title("WhatsApp"), 0);
        assert_eq!(count_from_title(""), 0);
        assert_eq!(count_from_title("() WhatsApp"), 0);
        assert_eq!(count_from_title("(x) WhatsApp"), 0);
        assert_eq!(count_from_title("(3 WhatsApp"), 0);
    }
}
