//! Works around a crash in NVIDIA's driver. WebKit's GL video sink converts
//! each video frame with GStreamer's OpenGL code, and on the NVIDIA EGL
//! driver (615.71.09) that kills the web process with SIGFPE, again on every
//! reload. Without that sink WebKit takes the frames from GStreamer in
//! memory; pages, pictures and animations stay GPU accelerated and smooth.

use std::path::Path;

const DISABLE_GL_SINK: &str = "WEBKIT_GST_DISABLE_GL_SINK";
const NVIDIA_DRIVER: &str = "/proc/driver/nvidia/version";

/// Must run first in `main`, before any thread starts: web processes
/// inherit the environment. A value the user set themselves is kept.
pub fn prepare() {
    if needs_workaround(
        std::env::var_os(DISABLE_GL_SINK).is_some(),
        Path::new(NVIDIA_DRIVER),
    ) {
        // SAFETY: called at the start of `main`, while the process has one thread.
        unsafe { std::env::set_var(DISABLE_GL_SINK, "1") };
    }
}

fn needs_workaround(already_set: bool, nvidia_driver: &Path) -> bool {
    !already_set && nvidia_driver.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_for_nvidia_and_never_over_the_users_choice() {
        let present = Path::new(env!("CARGO_MANIFEST_DIR"));
        let missing = Path::new("/nonexistent/nvidia/version");
        assert!(needs_workaround(false, present));
        assert!(!needs_workaround(true, present));
        assert!(!needs_workaround(false, missing));
    }
}
