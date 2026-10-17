//! Works around a crash in NVIDIA's driver. When a page draws a video frame
//! onto a canvas (WhatsApp does for video previews), WebKit converts it on
//! the GPU, and with WebKit's DMA-BUF renderer that makes the NVIDIA EGL
//! driver kill the web process with SIGFPE, again on every reload. Without
//! that renderer WebKit stays hardware accelerated, and videos, including
//! the animated reactions, still play on the GPU.

use std::path::Path;

const DISABLE_DMABUF_RENDERER: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
const NVIDIA_DRIVER: &str = "/proc/driver/nvidia/version";

/// Must run first in `main`, before any thread starts: web processes
/// inherit the environment. A value the user set themselves is kept.
pub fn prepare() {
    if needs_workaround(
        std::env::var_os(DISABLE_DMABUF_RENDERER).is_some(),
        Path::new(NVIDIA_DRIVER),
    ) {
        // SAFETY: called at the start of `main`, while the process has one thread.
        unsafe { std::env::set_var(DISABLE_DMABUF_RENDERER, "1") };
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
