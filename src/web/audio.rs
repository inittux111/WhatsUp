//! Keeps videos from playing silently. WebKit hands each video's volume to
//! its audio stream and takes the stream's volume back, while PipeWire
//! remembers the last volume per app and restores it on every new stream.
//! So once a preview played at volume 0, every later video started at 0 too.
//! With restoring off, each video's own volume decides.

const PULSE_PROP: &str = "PULSE_PROP";
const NO_RESTORE: &str = "state.restore-props=false";

/// Must run first in `main`, before any thread starts: web processes
/// inherit the environment. Properties the user set themselves are kept.
pub fn prepare() {
    let current = std::env::var(PULSE_PROP).ok();
    if let Some(value) = with_no_restore(current.as_deref()) {
        // SAFETY: called at the start of `main`, while the process has one thread.
        unsafe { std::env::set_var(PULSE_PROP, value) };
    }
}

/// `PULSE_PROP` with restoring turned off, or `None` when it needs no change.
fn with_no_restore(current: Option<&str>) -> Option<String> {
    match current.map(str::trim) {
        None | Some("") => Some(NO_RESTORE.to_string()),
        Some(props) if props.contains("state.restore-props") => None,
        Some(props) => Some(format!("{props} {NO_RESTORE}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_off_restoring_and_keeps_the_users_properties() {
        assert_eq!(with_no_restore(None).as_deref(), Some(NO_RESTORE));
        assert_eq!(with_no_restore(Some(" ")).as_deref(), Some(NO_RESTORE));
        assert_eq!(
            with_no_restore(Some("media.role=video")).as_deref(),
            Some("media.role=video state.restore-props=false")
        );
        assert_eq!(with_no_restore(Some("state.restore-props=true")), None);
    }
}
